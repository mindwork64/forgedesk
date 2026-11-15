use rusqlite::{Connection, OptionalExtension, Row};

use crate::domain::pull_link::PullLink;
use crate::domain::time::now_rfc3339;
use crate::error::{CoreError, CoreResult};

#[allow(clippy::too_many_arguments)]
pub fn link(
    conn: &Connection,
    workspace_id: &str,
    issue_key: &str,
    provider: &str,
    account_id: &str,
    repository_id: &str,
    pull_number: i64,
    pull_url: &str,
    title: &str,
    state: &str,
) -> CoreResult<PullLink> {
    let issue = conn
        .query_row(
            "SELECT id, key, title FROM issue \
             WHERE workspace_id = ?1 AND upper(key) = upper(?2) AND deleted_at IS NULL",
            rusqlite::params![workspace_id, issue_key.trim()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| CoreError::not_found("Issue"))?;
    let id = uuid::Uuid::now_v7().to_string();
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO pull_request_link \
         (id, local_issue_id, provider, account_id, repository_id, pull_number, \
          pull_url, title, state, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10) \
         ON CONFLICT(provider, account_id, repository_id, pull_number) DO UPDATE SET \
          local_issue_id = excluded.local_issue_id, pull_url = excluded.pull_url, \
          title = excluded.title, state = excluded.state, updated_at = excluded.updated_at",
        rusqlite::params![
            id,
            issue.0,
            provider,
            account_id,
            repository_id,
            pull_number,
            pull_url,
            title,
            state,
            now,
        ],
    )?;
    get(
        conn,
        workspace_id,
        provider,
        account_id,
        repository_id,
        pull_number,
    )?
    .ok_or_else(|| CoreError::internal("Pull request link was not persisted."))
}

pub fn get(
    conn: &Connection,
    workspace_id: &str,
    provider: &str,
    account_id: &str,
    repository_id: &str,
    pull_number: i64,
) -> CoreResult<Option<PullLink>> {
    Ok(conn
        .query_row(
            "SELECT l.id, l.local_issue_id, i.key, i.title, l.provider, l.account_id, \
                    l.repository_id, l.pull_number, l.pull_url, l.title, l.state, \
                    l.created_at, l.updated_at \
             FROM pull_request_link l JOIN issue i ON i.id = l.local_issue_id \
             WHERE i.workspace_id = ?1 AND l.provider = ?2 AND l.account_id = ?3 \
               AND l.repository_id = ?4 AND l.pull_number = ?5",
            rusqlite::params![
                workspace_id,
                provider,
                account_id,
                repository_id,
                pull_number
            ],
            read_link,
        )
        .optional()?)
}

pub fn unlink(
    conn: &Connection,
    workspace_id: &str,
    provider: &str,
    account_id: &str,
    repository_id: &str,
    pull_number: i64,
) -> CoreResult<()> {
    let changed = conn.execute(
        "DELETE FROM pull_request_link WHERE id IN (\
           SELECT l.id FROM pull_request_link l JOIN issue i ON i.id = l.local_issue_id \
           WHERE i.workspace_id = ?1 AND l.provider = ?2 AND l.account_id = ?3 \
             AND l.repository_id = ?4 AND l.pull_number = ?5)",
        rusqlite::params![
            workspace_id,
            provider,
            account_id,
            repository_id,
            pull_number
        ],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Pull request link"));
    }
    Ok(())
}

fn read_link(row: &Row<'_>) -> rusqlite::Result<PullLink> {
    Ok(PullLink {
        id: row.get(0)?,
        local_issue_id: row.get(1)?,
        issue_key: row.get(2)?,
        issue_title: row.get(3)?,
        provider: row.get(4)?,
        account_id: row.get(5)?,
        repository_id: row.get(6)?,
        pull_number: row.get(7)?,
        pull_url: row.get(8)?,
        title: row.get(9)?,
        state: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::domain::issue::{CreateIssueInput, IssueType, Priority};

    fn fixture() -> (tempfile::TempDir, Database, String, String) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("db");
        let workspace = db
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "Links"))
            .expect("workspace");
        let issue = db
            .with_conn(|conn| {
                crate::db::repositories::issues::create(
                    conn,
                    &workspace.id,
                    &CreateIssueInput {
                        title: "Linked issue".into(),
                        description: String::new(),
                        status_id: None,
                        priority: Priority::Medium,
                        issue_type: IssueType::Task,
                        estimate: None,
                        start_date: None,
                        due_date: None,
                        project_id: None,
                        parent_id: None,
                        label_names: vec![],
                    },
                )
            })
            .expect("issue");
        let key = db
            .with_conn(|conn| {
                Ok(
                    conn.query_row("SELECT key FROM issue WHERE id = ?1", [&issue], |row| {
                        row.get::<_, String>(0)
                    })?,
                )
            })
            .expect("key");
        (dir, db, workspace.id, key)
    }

    #[test]
    fn link_upsert_and_unlink_are_scoped() {
        let (_dir, db, workspace_id, issue_key) = fixture();
        let first = db
            .with_conn(|conn| {
                link(
                    conn,
                    &workspace_id,
                    &issue_key,
                    "github",
                    "github:1",
                    "o/r",
                    7,
                    "https://github.com/o/r/pull/7",
                    "Old title",
                    "open",
                )
            })
            .expect("link");
        assert_eq!(first.issue_key, issue_key);
        assert_eq!(first.pull_number, 7);

        let updated = db
            .with_conn(|conn| {
                link(
                    conn,
                    &workspace_id,
                    &issue_key,
                    "github",
                    "github:1",
                    "o/r",
                    7,
                    "https://github.com/o/r/pull/7",
                    "New title",
                    "merged",
                )
            })
            .expect("update");
        assert_eq!(updated.id, first.id);
        assert_eq!(updated.title, "New title");
        assert_eq!(updated.state, "merged");

        assert!(db
            .with_conn(|conn| { get(conn, &workspace_id, "github", "github:1", "o/r", 7,) })
            .expect("get")
            .is_some());
        db.with_conn(|conn| unlink(conn, &workspace_id, "github", "github:1", "o/r", 7))
            .expect("unlink");
        assert!(db
            .with_conn(|conn| { get(conn, &workspace_id, "github", "github:1", "o/r", 7,) })
            .expect("get")
            .is_none());
    }

    #[test]
    fn link_rejects_unknown_issue_key() {
        let (_dir, db, workspace_id, _key) = fixture();
        assert!(db
            .with_conn(|conn| {
                link(
                    conn,
                    &workspace_id,
                    "FD-999999",
                    "github",
                    "github:1",
                    "o/r",
                    1,
                    "u",
                    "t",
                    "open",
                )
            })
            .is_err());
    }
}
