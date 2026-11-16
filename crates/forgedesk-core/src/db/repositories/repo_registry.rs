use std::path::PathBuf;

use rusqlite::{Connection, OptionalExtension, Row};

use crate::domain::git::{Repository, RepositoryCheckout, RepositoryInput};
use crate::domain::issue::ProjectRef;
use crate::domain::time::now_rfc3339;
use crate::error::{CoreError, CoreResult};
use crate::git;

const BASE_SELECT: &str = "\
SELECT r.id, r.provider, r.host, r.external_id, r.owner, r.name, r.full_name, \
       r.description, r.visibility, r.default_branch, r.html_url, r.clone_url, \
       r.canonical_remote_url, r.is_archived, MIN(c.last_checked_at), \
       r.created_at, r.updated_at \
FROM repository r LEFT JOIN repository_checkout c ON c.repository_id = r.id";

#[derive(Debug, Clone)]
struct RepositoryBase {
    id: String,
    provider: String,
    host: String,
    external_id: Option<String>,
    owner: Option<String>,
    name: String,
    full_name: String,
    description: Option<String>,
    visibility: Option<String>,
    default_branch: Option<String>,
    html_url: Option<String>,
    clone_url: Option<String>,
    canonical_remote_url: Option<String>,
    is_archived: bool,
    last_checked_at: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedRemote {
    pub canonical: String,
    pub host: String,
    pub provider: String,
    pub owner: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct RemoteRepositoryInput {
    pub provider: String,
    pub host: String,
    pub external_id: Option<String>,
    pub owner: Option<String>,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub visibility: Option<String>,
    pub default_branch: Option<String>,
    pub html_url: Option<String>,
    pub clone_url: Option<String>,
    pub canonical_remote_url: Option<String>,
    pub is_archived: bool,
}

pub fn list(conn: &Connection, workspace_id: &str) -> CoreResult<Vec<Repository>> {
    let mut statement = conn.prepare(&format!(
        "{BASE_SELECT} WHERE r.workspace_id = ?1 \
         GROUP BY r.id ORDER BY r.updated_at DESC, r.full_name COLLATE NOCASE"
    ))?;
    let rows = statement.query_map([workspace_id], read_base)?;
    let bases = rows
        .map(|row| row.map_err(CoreError::from))
        .collect::<CoreResult<Vec<_>>>()?;
    bases.into_iter().map(|base| hydrate(conn, base)).collect()
}

pub fn get(conn: &Connection, workspace_id: &str, id: &str) -> CoreResult<Repository> {
    let base = conn
        .query_row(
            &format!("{BASE_SELECT} WHERE r.workspace_id = ?1 AND r.id = ?2 GROUP BY r.id"),
            rusqlite::params![workspace_id, id],
            read_base,
        )
        .optional()?
        .ok_or_else(|| CoreError::not_found("Repository"))?;
    hydrate(conn, base)
}

pub fn find_by_canonical(
    conn: &Connection,
    workspace_id: &str,
    canonical_remote_url: &str,
) -> CoreResult<Option<Repository>> {
    let id: Option<String> = conn
        .query_row(
            "SELECT id FROM repository WHERE workspace_id = ?1 AND canonical_remote_url = ?2",
            rusqlite::params![workspace_id, canonical_remote_url],
            |row| row.get(0),
        )
        .optional()?;
    id.map(|id| get(conn, workspace_id, &id)).transpose()
}

pub fn add_local(
    conn: &Connection,
    workspace_id: &str,
    input: &RepositoryInput,
) -> CoreResult<Repository> {
    let path = git::canonical_repo_path(&input.path)?;
    git::ensure_git_repository(&path)?;
    let path_text = path.to_string_lossy().to_string();
    if let Some(project_id) = &input.project_id {
        ensure_project(conn, workspace_id, project_id)?;
    }
    if checkout_exists(conn, workspace_id, &path_text)? {
        return Err(CoreError::conflict(
            "This repository checkout is already added.",
        ));
    }

    let remote_url = git::remote_url(&path);
    let normalized = remote_url.as_deref().and_then(normalize_remote);
    let existing = normalized.as_ref().and_then(|remote| {
        find_by_canonical(conn, workspace_id, &remote.canonical)
            .ok()
            .flatten()
    });
    let repository_id = if let Some(repository) = existing {
        repository.id
    } else {
        let name = input
            .name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .or_else(|| {
                normalized
                    .as_ref()
                    .map(|remote| remote.name.clone())
                    .or_else(|| {
                        path.file_name()
                            .map(|value| value.to_string_lossy().to_string())
                    })
            })
            .unwrap_or_else(|| "repository".to_owned());
        let full_name = normalized
            .as_ref()
            .map(|remote| format!("{}/{}", remote.owner, remote.name))
            .unwrap_or_else(|| name.clone());
        let owner = normalized.as_ref().map(|remote| remote.owner.clone());
        let default_branch = git::current_branch(&path).ok();
        let id = uuid::Uuid::now_v7().to_string();
        let now = now_rfc3339();
        conn.execute(
            "INSERT INTO repository
             (id, workspace_id, provider, host, external_id, owner, name, full_name,
              description, visibility, default_branch, html_url, clone_url,
              canonical_remote_url, is_archived, created_at, updated_at)
             VALUES (?1, ?2, 'local', 'local', NULL, ?3, ?4, ?5, NULL, NULL, ?6,
                     NULL, ?7, ?8, 0, ?9, ?9)",
            rusqlite::params![
                id,
                workspace_id,
                owner,
                name,
                full_name,
                default_branch,
                remote_url,
                normalized.as_ref().map(|remote| remote.canonical.clone()),
                now,
            ],
        )?;
        id
    };

    attach_checkout(conn, workspace_id, &repository_id, &path_text)?;
    if let Some(project_id) = &input.project_id {
        link(conn, workspace_id, repository_id.as_str(), project_id)?;
    }
    get(conn, workspace_id, &repository_id)
}

pub fn upsert_remote(
    conn: &Connection,
    workspace_id: &str,
    input: &RemoteRepositoryInput,
) -> CoreResult<Repository> {
    let existing = if let Some(external_id) = &input.external_id {
        find_by_external(
            conn,
            workspace_id,
            &input.provider,
            &input.host,
            external_id,
        )?
    } else {
        None
    };
    let existing = match existing {
        Some(repository) => Some(repository),
        None => input
            .canonical_remote_url
            .as_deref()
            .map(|canonical| find_by_canonical(conn, workspace_id, canonical))
            .transpose()?
            .flatten(),
    };
    let now = now_rfc3339();
    let repository_id = if let Some(repository) = existing {
        conn.execute(
            "UPDATE repository SET provider = ?1, host = ?2, external_id = ?3, \
             owner = COALESCE(?4, owner), name = ?5, full_name = ?6, \
             description = COALESCE(?7, description), visibility = COALESCE(?8, visibility), \
             default_branch = COALESCE(?9, default_branch), html_url = COALESCE(?10, html_url), \
             clone_url = COALESCE(?11, clone_url), \
             canonical_remote_url = COALESCE(?12, canonical_remote_url), \
             is_archived = ?13, updated_at = ?14 \
             WHERE id = ?15",
            rusqlite::params![
                input.provider,
                input.host,
                input.external_id,
                input.owner,
                input.name,
                input.full_name,
                input.description,
                input.visibility,
                input.default_branch,
                input.html_url,
                input.clone_url,
                input.canonical_remote_url,
                i64::from(input.is_archived),
                now,
                repository.id,
            ],
        )?;
        repository.id
    } else {
        let id = uuid::Uuid::now_v7().to_string();
        conn.execute(
            "INSERT INTO repository
             (id, workspace_id, provider, host, external_id, owner, name, full_name,
              description, visibility, default_branch, html_url, clone_url,
              canonical_remote_url, is_archived, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?16)",
            rusqlite::params![
                id,
                workspace_id,
                input.provider,
                input.host,
                input.external_id,
                input.owner,
                input.name,
                input.full_name,
                input.description,
                input.visibility,
                input.default_branch,
                input.html_url,
                input.clone_url,
                input.canonical_remote_url,
                i64::from(input.is_archived),
                now,
            ],
        )?;
        id
    };
    get(conn, workspace_id, &repository_id)
}

pub fn attach_checkout_checked(
    conn: &Connection,
    workspace_id: &str,
    repository_id: &str,
    raw_path: &str,
) -> CoreResult<RepositoryCheckout> {
    let repository = get(conn, workspace_id, repository_id)?;
    let path = git::canonical_repo_path(raw_path)?;
    git::ensure_git_repository(&path)?;
    let path_text = path.to_string_lossy().to_string();
    if let Some(expected) = &repository.canonical_remote_url {
        let remote = git::remote_url(&path).ok_or_else(|| {
            CoreError::validation(
                "path",
                "The checkout has no origin remote and cannot be matched to this repository.",
            )
        })?;
        let normalized = normalize_remote(&remote).ok_or_else(|| {
            CoreError::validation("path", "The checkout origin remote is not recognized.")
        })?;
        if &normalized.canonical != expected {
            return Err(CoreError::validation(
                "path",
                "The checkout origin does not match this repository.",
            ));
        }
    }
    attach_checkout(conn, workspace_id, repository_id, &path_text)
}

pub fn attach_checkout(
    conn: &Connection,
    workspace_id: &str,
    repository_id: &str,
    path: &str,
) -> CoreResult<RepositoryCheckout> {
    if checkout_exists(conn, workspace_id, path)? {
        return Err(CoreError::conflict(
            "This repository checkout is already added.",
        ));
    }
    get(conn, workspace_id, repository_id)?;
    let existing: i64 = conn.query_row(
        "SELECT count(*) FROM repository_checkout WHERE repository_id = ?1",
        [repository_id],
        |row| row.get(0),
    )?;
    if existing > 0 {
        return Err(CoreError::conflict(
            "This repository already has a local checkout; detach it before attaching another.",
        ));
    }
    let id = uuid::Uuid::now_v7().to_string();
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO repository_checkout
         (id, workspace_id, repository_id, path, last_checked_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?5)",
        rusqlite::params![id, workspace_id, repository_id, path, now],
    )?;
    checkout(conn, workspace_id, &id)
}

pub fn checkout(
    conn: &Connection,
    workspace_id: &str,
    checkout_id: &str,
) -> CoreResult<RepositoryCheckout> {
    conn.query_row(
        "SELECT id, repository_id, path, last_checked_at, created_at, updated_at \
         FROM repository_checkout WHERE workspace_id = ?1 AND id = ?2",
        rusqlite::params![workspace_id, checkout_id],
        read_checkout,
    )
    .optional()?
    .ok_or_else(|| CoreError::not_found("Repository checkout"))
}

pub fn detach_checkout(conn: &Connection, workspace_id: &str, checkout_id: &str) -> CoreResult<()> {
    let changed = conn.execute(
        "DELETE FROM repository_checkout WHERE workspace_id = ?1 AND id = ?2",
        rusqlite::params![workspace_id, checkout_id],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Repository checkout"));
    }
    Ok(())
}

pub fn link(
    conn: &Connection,
    workspace_id: &str,
    repository_id: &str,
    project_id: &str,
) -> CoreResult<Repository> {
    ensure_project(conn, workspace_id, project_id)?;
    get(conn, workspace_id, repository_id)?;
    conn.execute(
        "INSERT OR IGNORE INTO repository_project
         (workspace_id, repository_id, project_id, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![workspace_id, repository_id, project_id, now_rfc3339()],
    )?;
    get(conn, workspace_id, repository_id)
}

pub fn unlink(
    conn: &Connection,
    workspace_id: &str,
    repository_id: &str,
    project_id: &str,
) -> CoreResult<Repository> {
    get(conn, workspace_id, repository_id)?;
    conn.execute(
        "DELETE FROM repository_project \
         WHERE workspace_id = ?1 AND repository_id = ?2 AND project_id = ?3",
        rusqlite::params![workspace_id, repository_id, project_id],
    )?;
    get(conn, workspace_id, repository_id)
}

pub fn list_for_project(
    conn: &Connection,
    workspace_id: &str,
    project_id: &str,
) -> CoreResult<Vec<Repository>> {
    ensure_project(conn, workspace_id, project_id)?;
    let mut statement = conn.prepare(&format!(
        "{BASE_SELECT} JOIN repository_project rp ON rp.repository_id = r.id \
         WHERE r.workspace_id = ?1 AND rp.project_id = ?2 GROUP BY r.id \
         ORDER BY r.updated_at DESC, r.full_name COLLATE NOCASE"
    ))?;
    let rows = statement.query_map(rusqlite::params![workspace_id, project_id], read_base)?;
    let bases = rows
        .map(|row| row.map_err(CoreError::from))
        .collect::<CoreResult<Vec<_>>>()?;
    bases.into_iter().map(|base| hydrate(conn, base)).collect()
}

pub fn touch(
    conn: &Connection,
    workspace_id: &str,
    repository_id: &str,
    checkout_id: &str,
) -> CoreResult<()> {
    let now = now_rfc3339();
    let changed = conn.execute(
        "UPDATE repository_checkout SET last_checked_at = ?1, updated_at = ?1 \
         WHERE workspace_id = ?2 AND id = ?3 AND repository_id = ?4",
        rusqlite::params![now, workspace_id, checkout_id, repository_id],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Repository checkout"));
    }
    conn.execute(
        "UPDATE repository SET updated_at = ?1 WHERE workspace_id = ?2 AND id = ?3",
        rusqlite::params![now, workspace_id, repository_id],
    )?;
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

pub fn checkout_path(repository: &Repository) -> CoreResult<PathBuf> {
    repository
        .checkouts
        .first()
        .map(|checkout| PathBuf::from(&checkout.path))
        .ok_or_else(|| {
            CoreError::unsupported(
                "This repository has no local checkout. Attach a checkout first.",
            )
        })
}

pub fn normalize_remote(raw: &str) -> Option<NormalizedRemote> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let (host, path) = if let Some(rest) = raw.strip_prefix("git@") {
        let (host, path) = rest.split_once(':')?;
        (host.to_owned(), path.to_owned())
    } else if let Some(rest) = raw.strip_prefix("ssh://") {
        let rest = rest.split_once('@').map(|(_, value)| value).unwrap_or(rest);
        let (host, path) = rest.split_once('/')?;
        (host.to_owned(), path.to_owned())
    } else {
        let rest = raw
            .strip_prefix("https://")
            .or_else(|| raw.strip_prefix("http://"))?;
        let rest = rest.split(['?', '#']).next()?;
        let (host, path) = rest.split_once('/')?;
        (host.to_owned(), path.to_owned())
    };
    let host = host.trim().trim_end_matches('/').to_ascii_lowercase();
    let path = path
        .trim()
        .trim_matches('/')
        .trim_end_matches(".git")
        .trim_matches('/');
    let mut parts = path.split('/');
    let owner = parts.next()?.trim();
    let name = parts.next()?.trim();
    if host.is_empty()
        || owner.is_empty()
        || name.is_empty()
        || parts.next().is_some()
        || owner == "."
        || name == "."
    {
        return None;
    }
    let provider = match host.as_str() {
        "github.com" => "github",
        "gitlab.com" => "gitlab",
        "gitea.com" => "gitea",
        _ => "local",
    };
    Some(NormalizedRemote {
        canonical: format!(
            "https://{}/{}",
            host,
            format!("{owner}/{name}").to_ascii_lowercase()
        ),
        host,
        provider: provider.into(),
        owner: owner.into(),
        name: name.into(),
    })
}

fn ensure_project(conn: &Connection, workspace_id: &str, project_id: &str) -> CoreResult<()> {
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM project WHERE workspace_id = ?1 AND id = ?2",
            rusqlite::params![workspace_id, project_id],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);
    if !exists {
        return Err(CoreError::not_found("Project"));
    }
    Ok(())
}

fn checkout_exists(conn: &Connection, workspace_id: &str, path: &str) -> CoreResult<bool> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM repository_checkout WHERE workspace_id = ?1 AND path = ?2",
            rusqlite::params![workspace_id, path],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false))
}

fn find_by_external(
    conn: &Connection,
    workspace_id: &str,
    provider: &str,
    host: &str,
    external_id: &str,
) -> CoreResult<Option<Repository>> {
    let id: Option<String> = conn
        .query_row(
            "SELECT id FROM repository WHERE workspace_id = ?1 AND provider = ?2 \
             AND host = ?3 AND external_id = ?4",
            rusqlite::params![workspace_id, provider, host, external_id],
            |row| row.get(0),
        )
        .optional()?;
    id.map(|id| get(conn, workspace_id, &id)).transpose()
}

fn hydrate(conn: &Connection, base: RepositoryBase) -> CoreResult<Repository> {
    let mut checkouts = conn.prepare(
        "SELECT id, repository_id, path, last_checked_at, created_at, updated_at \
         FROM repository_checkout WHERE repository_id = ?1 ORDER BY created_at",
    )?;
    let checkouts = checkouts
        .query_map([&base.id], read_checkout)?
        .collect::<Result<Vec<_>, _>>()?;
    let mut projects = conn.prepare(
        "SELECT p.id, p.name FROM project p \
         JOIN repository_project rp ON rp.project_id = p.id \
         WHERE rp.repository_id = ?1 AND p.archived_at IS NULL \
         ORDER BY p.name COLLATE NOCASE",
    )?;
    let projects = projects
        .query_map([&base.id], |row| {
            Ok(ProjectRef {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Repository {
        id: base.id,
        provider: base.provider,
        host: base.host,
        external_id: base.external_id,
        owner: base.owner,
        name: base.name,
        full_name: base.full_name,
        description: base.description,
        visibility: base.visibility,
        default_branch: base.default_branch,
        html_url: base.html_url,
        clone_url: base.clone_url,
        canonical_remote_url: base.canonical_remote_url,
        is_archived: base.is_archived,
        checkouts,
        projects,
        last_checked_at: base.last_checked_at,
        created_at: base.created_at,
        updated_at: base.updated_at,
    })
}

fn read_base(row: &Row<'_>) -> rusqlite::Result<RepositoryBase> {
    Ok(RepositoryBase {
        id: row.get(0)?,
        provider: row.get(1)?,
        host: row.get(2)?,
        external_id: row.get(3)?,
        owner: row.get(4)?,
        name: row.get(5)?,
        full_name: row.get(6)?,
        description: row.get(7)?,
        visibility: row.get(8)?,
        default_branch: row.get(9)?,
        html_url: row.get(10)?,
        clone_url: row.get(11)?,
        canonical_remote_url: row.get(12)?,
        is_archived: row.get::<_, i64>(13)? != 0,
        last_checked_at: row.get(14)?,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
    })
}

fn read_checkout(row: &Row<'_>) -> rusqlite::Result<RepositoryCheckout> {
    Ok(RepositoryCheckout {
        id: row.get(0)?,
        repository_id: row.get(1)?,
        path: row.get(2)?,
        last_checked_at: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub fn set_selected(
    conn: &Connection,
    workspace_id: &str,
    repository_id: Option<&str>,
) -> CoreResult<Option<Repository>> {
    match repository_id {
        Some(id) => {
            let repository = get(conn, workspace_id, id)?;
            conn.execute(
                "INSERT INTO app_setting (key, value, updated_at) VALUES ('repository.selected_id', ?1, ?2) \
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                rusqlite::params![id, now_rfc3339()],
            )?;
            Ok(Some(repository))
        }
        None => {
            conn.execute(
                "DELETE FROM app_setting WHERE key = 'repository.selected_id'",
                [],
            )?;
            Ok(None)
        }
    }
}

pub fn selected(conn: &Connection, workspace_id: &str) -> CoreResult<Option<Repository>> {
    let id: Option<String> = conn
        .query_row(
            "SELECT value FROM app_setting WHERE key = 'repository.selected_id'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    match id {
        Some(id) => match get(conn, workspace_id, &id) {
            Ok(repository) => Ok(Some(repository)),
            Err(error) if error.category() == crate::error::ErrorCategory::NotFound => Ok(None),
            Err(error) => Err(error),
        },
        None => Ok(None),
    }
}

/// Normalize legacy `repository.remote_url` values after the v10 table rebuild.
/// Unknown or unparsable URLs remain local-only; we never invent provider identity.
pub fn reconcile_legacy_remotes(conn: &Connection) -> CoreResult<()> {
    let mut statement = conn.prepare(
        "SELECT id, clone_url FROM repository \
         WHERE provider = 'local' AND external_id IS NULL \
           AND canonical_remote_url IS NULL AND clone_url IS NOT NULL",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (id, raw) in rows {
        let Some(remote) = normalize_remote(&raw) else {
            continue;
        };
        let conflict: Option<String> = conn
            .query_row(
                "SELECT id FROM repository WHERE canonical_remote_url = ?1 AND id <> ?2",
                rusqlite::params![remote.canonical, id],
                |row| row.get(0),
            )
            .optional()?;
        if conflict.is_some() {
            continue;
        }
        conn.execute(
            "UPDATE repository SET provider = ?1, host = ?2, owner = ?3, \
             name = CASE WHEN trim(name) = '' THEN ?4 ELSE name END, \
             full_name = ?5, canonical_remote_url = ?6 WHERE id = ?7",
            rusqlite::params![
                remote.provider,
                remote.host,
                remote.owner,
                remote.name,
                format!("{}/{}", remote.owner, remote.name),
                remote.canonical,
                id,
            ],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod model_tests {
    use super::*;
    use crate::db::Database;
    use crate::domain::planning::{PlanningStatus, ProjectInput};

    fn project(db: &Database, workspace_id: &str, name: &str) -> String {
        db.with_conn(|conn| {
            crate::db::repositories::planning::create_project(
                conn,
                workspace_id,
                &ProjectInput {
                    name: name.into(),
                    status: Some(PlanningStatus::Active),
                    ..Default::default()
                },
            )
        })
        .expect("project")
        .id
    }

    #[test]
    fn normalizes_common_remote_forms_without_guessing_unknown_hosts() {
        let expected = "https://github.com/mindwork64/forgedesk";
        for raw in [
            "https://github.com/mindwork64/forgedesk",
            "https://github.com/mindwork64/forgedesk.git",
            "https://github.com/mindwork64/forgedesk/",
            "git@github.com:mindwork64/forgedesk.git",
            "ssh://git@github.com/mindwork64/forgedesk.git",
        ] {
            let normalized = normalize_remote(raw).expect(raw);
            assert_eq!(normalized.canonical, expected);
            assert_eq!(normalized.provider, "github");
        }
        let unknown = normalize_remote("git@git.example.test:team/repo.git").expect("unknown");
        assert_eq!(unknown.provider, "local");
        assert_eq!(unknown.host, "git.example.test");
        assert!(normalize_remote("not a remote").is_none());
    }

    #[test]
    fn rejects_cross_workspace_link_and_clears_stale_selection() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("db");
        let workspace = db
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "One"))
            .expect("workspace");
        let _project_id = project(&db, &workspace.id, "One project");
        let remote = RemoteRepositoryInput {
            provider: "github".into(),
            host: "github.com".into(),
            external_id: Some("9".into()),
            owner: Some("owner".into()),
            name: "repo".into(),
            full_name: "owner/repo".into(),
            description: None,
            visibility: None,
            default_branch: None,
            html_url: None,
            clone_url: None,
            canonical_remote_url: Some("https://github.com/owner/repo".into()),
            is_archived: false,
        };
        let repository = db
            .with_transaction(|tx| upsert_remote(tx, &workspace.id, &remote))
            .expect("repository");
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO workspace (id, name, key_prefix, created_at, updated_at)                  VALUES ('ws-2', 'Two', 'TWO', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                [],
            )?;
            Ok(())
        })
        .expect("second workspace");
        let second_project = project(&db, "ws-2", "Two project");
        assert!(db
            .with_transaction(|tx| link(tx, &workspace.id, &repository.id, &second_project))
            .is_err());
        db.with_transaction(|tx| set_selected(tx, &workspace.id, Some(&repository.id)))
            .expect("select");
        db.with_transaction(|tx| remove(tx, &workspace.id, &repository.id))
            .expect("remove");
        let selected = db
            .with_conn(|conn| selected(conn, &workspace.id))
            .expect("selected");
        assert!(selected.is_none());
    }

    #[test]
    fn remote_only_upsert_is_idempotent_and_preserves_links() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("db");
        let workspace = db
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "Repos"))
            .expect("workspace");
        let project_id = project(&db, &workspace.id, "Project");
        let input = RemoteRepositoryInput {
            provider: "github".into(),
            host: "github.com".into(),
            external_id: Some("123".into()),
            owner: Some("owner".into()),
            name: "repo".into(),
            full_name: "owner/repo".into(),
            description: Some("first".into()),
            visibility: Some("public".into()),
            default_branch: Some("main".into()),
            html_url: Some("https://github.com/owner/repo".into()),
            clone_url: Some("https://github.com/owner/repo.git".into()),
            canonical_remote_url: Some("https://github.com/owner/repo".into()),
            is_archived: false,
        };
        let first = db
            .with_transaction(|tx| upsert_remote(tx, &workspace.id, &input))
            .expect("first");
        db.with_transaction(|tx| link(tx, &workspace.id, &first.id, &project_id))
            .expect("link before rename");
        let second = db
            .with_transaction(|tx| {
                let mut renamed = input.clone();
                renamed.full_name = "owner/renamed".into();
                renamed.external_id = Some("123".into());
                upsert_remote(tx, &workspace.id, &renamed)
            })
            .expect("second");
        assert_eq!(first.id, second.id);
        assert_eq!(second.full_name, "owner/renamed");
        assert!(second.checkouts.is_empty());
        assert_eq!(second.projects.len(), 1, "rename preserves project links");
        db.with_conn(|conn| {
            conn.execute(
                "UPDATE project SET archived_at = ?1 WHERE id = ?2",
                rusqlite::params![now_rfc3339(), project_id],
            )?;
            Ok(())
        })
        .expect("archive project");
        let archived = db
            .with_conn(|conn| get(conn, &workspace.id, &first.id))
            .expect("archived link");
        assert!(
            archived.projects.is_empty(),
            "archived projects stay out of link picker"
        );
    }

    #[test]
    fn attach_checkout_requires_matching_remote_identity() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("db");
        let workspace = db
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "Repos"))
            .expect("workspace");
        let remote = RemoteRepositoryInput {
            provider: "github".into(),
            host: "github.com".into(),
            external_id: Some("777".into()),
            owner: Some("owner".into()),
            name: "repo".into(),
            full_name: "owner/repo".into(),
            description: None,
            visibility: None,
            default_branch: None,
            html_url: None,
            clone_url: None,
            canonical_remote_url: Some("https://github.com/owner/repo".into()),
            is_archived: false,
        };
        let repository = db
            .with_transaction(|tx| upsert_remote(tx, &workspace.id, &remote))
            .expect("remote");
        let checkout = dir.path().join("checkout");
        std::fs::create_dir_all(&checkout).expect("directory");
        assert!(std::process::Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(&checkout)
            .status()
            .expect("git")
            .success());
        assert!(std::process::Command::new("git")
            .args(["remote", "add", "origin", "git@github.com:owner/repo.git",])
            .current_dir(&checkout)
            .status()
            .expect("git")
            .success());
        db.with_conn(|conn| {
            attach_checkout_checked(
                conn,
                &workspace.id,
                &repository.id,
                &checkout.to_string_lossy(),
            )
            .map(|_| ())
        })
        .expect("matching remote");

        let mismatch = dir.path().join("mismatch");
        std::fs::create_dir_all(&mismatch).expect("directory");
        assert!(std::process::Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(&mismatch)
            .status()
            .expect("git")
            .success());
        assert!(std::process::Command::new("git")
            .args([
                "remote",
                "add",
                "origin",
                "https://github.com/other/repo.git"
            ])
            .current_dir(&mismatch)
            .status()
            .expect("git")
            .success());
        assert!(db
            .with_conn(|conn| {
                attach_checkout_checked(
                    conn,
                    &workspace.id,
                    &repository.id,
                    &mismatch.to_string_lossy(),
                )
                .map(|_| ())
            })
            .is_err());
    }

    #[test]
    fn local_checkout_and_project_links_survive_unlink() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().join("checkout");
        std::fs::create_dir_all(&repo).expect("directory");
        assert!(std::process::Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(&repo)
            .status()
            .expect("git")
            .success());
        let db = Database::open(dir.path().join("test.db")).expect("db");
        let workspace = db
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "Repos"))
            .expect("workspace");
        let project_id = project(&db, &workspace.id, "Project");
        let added = db
            .with_conn(|conn| {
                add_local(
                    conn,
                    &workspace.id,
                    &RepositoryInput {
                        path: repo.to_string_lossy().to_string(),
                        name: Some("Local".into()),
                        project_id: Some(project_id.clone()),
                    },
                )
            })
            .expect("add");
        assert_eq!(added.checkouts.len(), 1);
        let canonical = git::canonical_repo_path(&repo.to_string_lossy()).expect("canonical");
        assert_eq!(added.checkouts[0].path, canonical.to_string_lossy());
        let unlinked = db
            .with_transaction(|tx| unlink(tx, &workspace.id, &added.id, &project_id))
            .expect("unlink");
        assert!(unlinked.projects.is_empty());
        assert_eq!(unlinked.checkouts.len(), 1);
        db.with_conn(|conn| {
            git_overview(conn, &workspace.id, &added.id).expect("checkout remains usable");
            Ok(())
        })
        .expect("overview");
        db.with_conn(|conn| {
            let checkout_id = &unlinked.checkouts[0].id;
            detach_checkout(conn, &workspace.id, checkout_id)
        })
        .expect("detach");
        let after = db
            .with_conn(|conn| get(conn, &workspace.id, &added.id))
            .expect("still exists");
        assert!(after.checkouts.is_empty());
        assert!(db
            .with_conn(|conn| git_overview(conn, &workspace.id, &added.id))
            .is_err());
    }

    fn git_overview(conn: &Connection, workspace_id: &str, id: &str) -> CoreResult<()> {
        let repository = get(conn, workspace_id, id)?;
        let _ = checkout_path(&repository)?;
        Ok(())
    }
}
