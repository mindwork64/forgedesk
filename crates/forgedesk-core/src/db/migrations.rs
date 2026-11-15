use rusqlite::Connection;

use crate::domain::time::now_rfc3339;
use crate::error::{CoreError, CoreResult};

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

/// Ordered, append-only migration list. Never edit a published migration; add
/// a new one and bump `CURRENT_SCHEMA_VERSION`.
pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "init",
    sql: include_str!("migrations/0001_init.sql"),
}];

pub const CURRENT_SCHEMA_VERSION: i64 = 1;

pub fn current_schema_version(conn: &Connection) -> CoreResult<i64> {
    let version = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    Ok(version)
}

/// Apply all pending migrations transactionally. A database created by a newer
/// ForgeDesk is refused instead of being silently downgraded.
pub fn run(conn: &mut Connection) -> CoreResult<i64> {
    let current = current_schema_version(conn)?;

    if current > CURRENT_SCHEMA_VERSION {
        return Err(CoreError::startup(format!(
            "This workspace database uses schema version {current}, but this build supports \
             version {CURRENT_SCHEMA_VERSION}. Update ForgeDesk to open it."
        )));
    }

    for migration in MIGRATIONS.iter().filter(|m| m.version > current) {
        let tx = conn.transaction()?;
        tx.execute_batch(migration.sql)?;
        tx.execute(
            "INSERT INTO schema_migration (version, name, applied_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![migration.version, migration.name, now_rfc3339()],
        )?;
        // PRAGMA user_version is stored in the database header and participates
        // in the surrounding transaction.
        tx.pragma_update(None, "user_version", migration.version)?;
        tx.commit()?;
    }

    current_schema_version(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        conn.pragma_update(None, "foreign_keys", "ON")
            .expect("enable fk");
        conn
    }

    #[test]
    fn applies_migrations_idempotently() {
        let mut conn = connection();
        assert_eq!(run(&mut conn).expect("first run"), CURRENT_SCHEMA_VERSION);
        assert_eq!(run(&mut conn).expect("second run"), CURRENT_SCHEMA_VERSION);

        let applied: i64 = conn
            .query_row("SELECT count(*) FROM schema_migration", [], |row| {
                row.get(0)
            })
            .expect("count migrations");
        assert_eq!(applied, CURRENT_SCHEMA_VERSION);

        let workspace_table: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'workspace'",
                [],
                |row| row.get(0),
            )
            .expect("workspace table");
        assert_eq!(workspace_table, 1);
    }

    #[test]
    fn refuses_newer_schema() {
        let mut conn = connection();
        run(&mut conn).expect("migrate");
        conn.pragma_update(None, "user_version", 999)
            .expect("simulate newer schema");
        let error = run(&mut conn).expect_err("must refuse newer schema");
        assert_eq!(error.category(), crate::error::ErrorCategory::Startup);
    }
}
