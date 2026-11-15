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

    Ok(workspace)
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
}
