use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Row};

use crate::domain::git::{Repository, RepositoryInput};
use crate::domain::time::now_rfc3339;
use crate::error::{CoreError, CoreResult};
use crate::git;

const SELECT: &str = "\
SELECT r.id, r.name, r.path, r.provider, r.remote_url, r.default_branch, \
       r.project_id, p.name, r.last_checked_at, r.created_at, r.updated_at \
FROM repository r LEFT JOIN project p ON p.id = r.project_id";

pub fn list(conn: &Connection, workspace_id: &str) -> CoreResult<Vec<Repository>> {
    let mut statement = conn.prepare(&format!(
        "{SELECT} WHERE r.workspace_id = ?1 ORDER BY r.updated_at DESC, r.name COLLATE NOCASE"
    ))?;
    let rows = statement.query_map([workspace_id], read_repository)?;
    rows.map(|row| row.map_err(CoreError::from))
        .collect::<CoreResult<Vec<_>>>()
}

pub fn get(conn: &Connection, workspace_id: &str, id: &str) -> CoreResult<Repository> {
    conn.query_row(
        &format!("{SELECT} WHERE r.workspace_id = ?1 AND r.id = ?2"),
        rusqlite::params![workspace_id, id],
        read_repository,
    )
    .optional()?
    .ok_or_else(|| CoreError::not_found("Repository"))
}

pub fn add_local(
    conn: &Connection,
    workspace_id: &str,
    input: &RepositoryInput,
) -> CoreResult<Repository> {
    let path = git::canonical_repo_path(&input.path)?;
    git::ensure_git_repository(&path)?;
    let path_text = path.to_string_lossy().to_string();
    if conn
        .query_row(
            "SELECT 1 FROM repository WHERE workspace_id = ?1 AND path = ?2",
            rusqlite::params![workspace_id, path_text],
            |_| Ok(()),
        )
        .optional()?
        .is_some()
    {
        return Err(CoreError::conflict("This repository is already added."));
    }
    if let Some(project_id) = &input.project_id {
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM project WHERE workspace_id = ?1 AND id = ?2 AND archived_at IS NULL",
                rusqlite::params![workspace_id, project_id],
                |_| Ok(true),
            )
            .optional()?
            .unwrap_or(false);
        if !exists {
            return Err(CoreError::not_found("Project"));
        }
    }
    let name = input
        .name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| {
            path.file_name()
                .map(|value| value.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "repository".to_owned());
    let default_branch = git::current_branch(&path).ok();
    let remote_url = git::remote_url(&path);
    let id = uuid::Uuid::now_v7().to_string();
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO repository \
         (id, workspace_id, name, path, provider, remote_url, default_branch, project_id, \
          last_checked_at, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, 'local', ?5, ?6, ?7, ?8, ?8, ?8)",
        rusqlite::params![
            id,
            workspace_id,
            name,
            path_text,
            remote_url,
            default_branch,
            input.project_id,
            now,
        ],
    )?;
    get(conn, workspace_id, &id)
}

pub fn touch(conn: &Connection, workspace_id: &str, id: &str) -> CoreResult<()> {
    let now = now_rfc3339();
    let changed = conn.execute(
        "UPDATE repository SET last_checked_at = ?1, updated_at = ?1 \
         WHERE workspace_id = ?2 AND id = ?3",
        rusqlite::params![now, workspace_id, id],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Repository"));
    }
    Ok(())
}

pub fn remove(conn: &Connection, workspace_id: &str, id: &str) -> CoreResult<()> {
    let changed = conn.execute(
        "DELETE FROM repository WHERE workspace_id = ?1 AND id = ?2",
        rusqlite::params![workspace_id, id],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Repository"));
    }
    Ok(())
}

pub fn path_of(repository: &Repository) -> &Path {
    Path::new(&repository.path)
}

fn read_repository(row: &Row<'_>) -> rusqlite::Result<Repository> {
    Ok(Repository {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        provider: row.get(3)?,
        remote_url: row.get(4)?,
        default_branch: row.get(5)?,
        project_id: row.get(6)?,
        project_name: row.get(7)?,
        last_checked_at: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::domain::git::RepositoryInput;
    use std::process::Command;

    fn git(dir: &Path, args: &[&str]) {
        assert!(Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_TERMINAL_PROMPT", "0")
            .status()
            .expect("git")
            .success());
    }

    fn fixture() -> (tempfile::TempDir, Database, String, std::path::PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().join("repo with space ü");
        std::fs::create_dir_all(&repo).expect("repo dir");
        git(&repo, &["init", "-b", "main"]);
        git(&repo, &["config", "user.email", "test@example.com"]);
        git(&repo, &["config", "user.name", "Test"]);
        std::fs::write(repo.join("README.md"), "hello\n").expect("file");
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-m", "initial"]);
        let db = Database::open(dir.path().join("test.db")).expect("db");
        let workspace = db
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "Git"))
            .expect("workspace");
        (dir, db, workspace.id, repo)
    }

    #[test]
    fn adds_lists_and_rejects_duplicate_local_repository() {
        let (_dir, db, workspace_id, repo) = fixture();
        let added = db
            .with_conn(|conn| {
                add_local(
                    conn,
                    &workspace_id,
                    &RepositoryInput {
                        path: repo.to_string_lossy().to_string(),
                        name: None,
                        project_id: None,
                    },
                )
            })
            .expect("add");
        assert_eq!(added.default_branch.as_deref(), Some("main"));
        assert_eq!(added.name, "repo with space ü");
        let list = db
            .with_conn(|conn| list(conn, &workspace_id))
            .expect("list");
        assert_eq!(list.len(), 1);
        assert!(db
            .with_conn(|conn| {
                add_local(
                    conn,
                    &workspace_id,
                    &RepositoryInput {
                        path: repo.to_string_lossy().to_string(),
                        name: None,
                        project_id: None,
                    },
                )
            })
            .is_err());
    }
}
