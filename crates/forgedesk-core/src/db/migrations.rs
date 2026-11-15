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
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "init",
        sql: include_str!("migrations/0001_init.sql"),
    },
    Migration {
        version: 2,
        name: "issues",
        sql: include_str!("migrations/0002_issues.sql"),
    },
    Migration {
        version: 3,
        name: "planning",
        sql: include_str!("migrations/0003_planning.sql"),
    },
    Migration {
        version: 4,
        name: "git_budget",
        sql: include_str!("migrations/0004_git_budget.sql"),
    },
    Migration {
        version: 5,
        name: "sync",
        sql: include_str!("migrations/0005_sync.sql"),
    },
    Migration {
        version: 6,
        name: "pull_links",
        sql: include_str!("migrations/0006_pull_links.sql"),
    },
    Migration {
        version: 7,
        name: "inbox",
        sql: include_str!("migrations/0007_inbox.sql"),
    },
];

pub const CURRENT_SCHEMA_VERSION: i64 = 7;

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
    fn upgrades_a_v1_workspace_with_defaults() {
        let mut conn = connection();
        let tx = conn.transaction().expect("tx");
        tx.execute_batch(MIGRATIONS[0].sql).expect("v1 schema");
        tx.pragma_update(None, "user_version", 1)
            .expect("v1 version");
        tx.execute(
            "INSERT INTO workspace (id, name, key_prefix, created_at, updated_at) \
             VALUES ('ws-1', 'Workspace', 'FD', '2026-11-15T00:00:00.000Z', '2026-11-15T00:00:00.000Z')",
            [],
        )
        .expect("workspace");
        tx.commit().expect("commit");

        assert_eq!(run(&mut conn).expect("migrate to current"), 7);

        let workflows: i64 = conn
            .query_row("SELECT count(*) FROM workflow", [], |row| row.get(0))
            .expect("workflows");
        let statuses: i64 = conn
            .query_row("SELECT count(*) FROM workflow_status", [], |row| row.get(0))
            .expect("statuses");
        let counter: i64 = conn
            .query_row(
                "SELECT next_number FROM workspace_key_counter WHERE workspace_id = 'ws-1'",
                [],
                |row| row.get(0),
            )
            .expect("counter");
        assert_eq!(workflows, 1);
        assert_eq!(statuses, 5);
        assert_eq!(counter, 1);
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
