use rusqlite::{Connection, OptionalExtension, Row};

use crate::domain::time::now_rfc3339;
use crate::domain::{Workspace, DEFAULT_KEY_PREFIX};
use crate::error::{CoreError, CoreResult};

const COLUMNS: &str = "id, name, key_prefix, created_at, updated_at";

fn map_row(row: &Row<'_>) -> rusqlite::Result<Workspace> {
    Ok(Workspace {
        id: row.get(0)?,
        name: row.get(1)?,
        key_prefix: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

pub fn load_first(conn: &Connection) -> CoreResult<Option<Workspace>> {
    let sql = format!("SELECT {COLUMNS} FROM workspace ORDER BY created_at ASC LIMIT 1");
    let workspace = conn.query_row(&sql, [], map_row).optional()?;
    Ok(workspace)
}

pub fn count(conn: &Connection) -> CoreResult<i64> {
    let count = conn.query_row("SELECT count(*) FROM workspace", [], |row| row.get(0))?;
    Ok(count)
}

/// Create the single v1 workspace. The transaction ensures two concurrent
/// onboarding attempts cannot create two workspaces.
pub fn create_initial(conn: &Connection, name: &str) -> CoreResult<Workspace> {
    let name = Workspace::validate_name(name)?;
    if count(conn)? > 0 {
        return Err(CoreError::conflict(
            "A workspace already exists in this installation.",
        ));
    }

    let now = now_rfc3339();
    let workspace = Workspace {
        id: uuid::Uuid::now_v7().to_string(),
        name,
        key_prefix: DEFAULT_KEY_PREFIX.to_owned(),
        created_at: now.clone(),
        updated_at: now,
    };

    conn.execute(
        "INSERT INTO workspace (id, name, key_prefix, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            workspace.id,
            workspace.name,
            workspace.key_prefix,
            workspace.created_at,
            workspace.updated_at
        ],
    )?;

    super::workflow::seed_defaults(conn, &workspace.id)?;
    conn.execute(
        "INSERT INTO workspace_key_counter (workspace_id, next_number) VALUES (?1, 1)",
        [&workspace.id],
    )?;

    Ok(workspace)
}

/// Allocate the next human key inside the create-issue transaction.
pub fn next_issue_key(conn: &Connection, workspace_id: &str) -> CoreResult<(i64, String)> {
    conn.execute(
        "INSERT OR IGNORE INTO workspace_key_counter (workspace_id, next_number) \
         SELECT id, 1 FROM workspace WHERE id = ?1",
        [workspace_id],
    )?;

    let (prefix, number) = conn
        .query_row(
            "SELECT w.key_prefix, c.next_number FROM workspace w \
             JOIN workspace_key_counter c ON c.workspace_id = w.id WHERE w.id = ?1",
            [workspace_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => CoreError::not_found("Workspace"),
            other => CoreError::from(other),
        })?;

    conn.execute(
        "UPDATE workspace_key_counter SET next_number = next_number + 1 \
         WHERE workspace_id = ?1",
        [workspace_id],
    )?;

    Ok((number, format!("{prefix}-{number}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn db() -> (tempfile::TempDir, Database) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("open");
        (dir, db)
    }

    #[test]
    fn creates_only_one_workspace() {
        let (_dir, db) = db();
        let first = db
            .with_conn(|conn| create_initial(conn, "  My Workspace  "))
            .expect("first workspace");
        assert_eq!(first.name, "My Workspace");
        assert_eq!(first.key_prefix, "FD");

        let error = db
            .with_conn(|conn| create_initial(conn, "Second"))
            .expect_err("second workspace must conflict");
        assert_eq!(error.category(), crate::error::ErrorCategory::Conflict);
    }

    #[test]
    fn rejects_invalid_names() {
        let (_dir, db) = db();
        for invalid in ["", "   ", "bad\nname"] {
            let error = db
                .with_conn(|conn| create_initial(conn, invalid))
                .expect_err("invalid name");
            assert_eq!(error.category(), crate::error::ErrorCategory::Validation);
        }
    }

    #[test]
    fn creates_default_workflow_counter_and_keys() {
        let (_dir, db) = db();
        let workspace = db
            .with_conn(|conn| create_initial(conn, "My Workspace"))
            .expect("workspace");
        let statuses = db
            .with_conn(|conn| crate::db::repositories::workflow::statuses(conn, &workspace.id))
            .expect("statuses");
        assert_eq!(statuses.len(), 5);

        let (number, key) = db
            .with_conn(|conn| next_issue_key(conn, &workspace.id))
            .expect("first key");
        assert_eq!(number, 1);
        assert_eq!(key, "FD-1");
        let (number, key) = db
            .with_conn(|conn| next_issue_key(conn, &workspace.id))
            .expect("second key");
        assert_eq!(number, 2);
        assert_eq!(key, "FD-2");
    }
}
