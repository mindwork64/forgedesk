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
    Migration {
        version: 8,
        name: "automations",
        sql: include_str!("migrations/0008_automations.sql"),
    },
    Migration {
        version: 9,
        name: "dashboards",
        sql: include_str!("migrations/0009_dashboards.sql"),
    },
    Migration {
        version: 10,
        name: "repositories",
        sql: include_str!("migrations/0010_repository_model.sql"),
    },
    Migration {
        version: 11,
        name: "project_issue_key_prefix",
        sql: include_str!("migrations/0011_project_issue_key_prefix.sql"),
    },
];

pub const CURRENT_SCHEMA_VERSION: i64 = 11;

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
        if migration.version == 10 {
            crate::db::repositories::repo_registry::reconcile_legacy_remotes(&tx)?;
        }
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

    fn apply_through(conn: &mut Connection, version: i64) {
        let current = current_schema_version(conn).expect("current version");
        for migration in MIGRATIONS
            .iter()
            .filter(|m| m.version > current && m.version <= version)
        {
            let tx = conn.transaction().expect("tx");
            tx.execute_batch(migration.sql).expect("apply");
            tx.execute(
                "INSERT INTO schema_migration (version, name, applied_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![migration.version, migration.name, now_rfc3339()],
            )
            .expect("record");
            tx.pragma_update(None, "user_version", migration.version)
                .expect("version");
            tx.commit().expect("commit");
        }
    }

    #[test]
    fn upgrades_v5_database_to_current_without_data_loss() {
        let mut conn = connection();
        let now = "2026-01-01T00:00:00Z";

        // A real v5 installation was created after migrations had already run:
        // workspace/app settings exist before v2 seeds the workflow.
        apply_through(&mut conn, 1);
        conn.execute(
            "INSERT INTO workspace (id, name, key_prefix, created_at, updated_at) \
             VALUES ('ws-1', 'Upgrade', 'FD', ?1, ?1)",
            [now],
        )
        .expect("workspace");
        conn.execute(
            "INSERT INTO app_setting (key, value, updated_at) VALUES ('theme', '\"dark\"', ?1)",
            [now],
        )
        .expect("setting");

        apply_through(&mut conn, 2);
        crate::db::repositories::workflow::seed_defaults(&conn, "ws-1").expect("default workflow");
        conn.execute(
            "UPDATE workspace_key_counter SET next_number = 2 WHERE workspace_id = 'ws-1'",
            [],
        )
        .expect("key counter");
        apply_through(&mut conn, 5);

        let workflow_id: String = conn
            .query_row(
                "SELECT id FROM workflow WHERE workspace_id = 'ws-1' AND is_default = 1",
                [],
                |row| row.get(0),
            )
            .expect("default workflow");
        let status_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM workflow_status WHERE workflow_id = ?1",
                [&workflow_id],
                |row| row.get(0),
            )
            .expect("statuses");
        assert_eq!(status_count, 5);
        let todo_status: String = conn
            .query_row(
                "SELECT id FROM workflow_status WHERE workflow_id = ?1 AND system_key = 'todo'",
                [&workflow_id],
                |row| row.get(0),
            )
            .expect("todo status");
        let next_number: i64 = conn
            .query_row(
                "SELECT next_number FROM workspace_key_counter WHERE workspace_id = 'ws-1'",
                [],
                |row| row.get(0),
            )
            .expect("counter");
        assert_eq!(next_number, 2);

        let project_id = "project-1";
        let issue_id = "issue-1";
        conn.execute(
            "INSERT INTO project (id, workspace_id, name, description, status, priority, \
             created_at, updated_at) VALUES (?1, 'ws-1', 'Release', 'release project', \
             'active', 'high', ?2, ?2)",
            rusqlite::params![project_id, now],
        )
        .expect("project");
        conn.execute(
            "INSERT INTO issue (id, workspace_id, key_number, key, title, description, \
             status_id, priority, issue_type, project_id, created_at, updated_at) \
             VALUES (?1, 'ws-1', 1, 'FD-1', 'Preserved issue', 'body', ?2, 'high', \
             'task', ?3, ?4, ?4)",
            rusqlite::params![issue_id, todo_status, project_id, now],
        )
        .expect("issue");
        conn.execute(
            "INSERT INTO comment (id, issue_id, body, author, created_at, updated_at) \
             VALUES ('comment-1', ?1, 'preserved comment', 'local', ?2, ?2)",
            rusqlite::params![issue_id, now],
        )
        .expect("comment");
        conn.execute(
            "INSERT INTO document (id, workspace_id, title, body, project_id, created_at, \
             updated_at) VALUES ('doc-1', 'ws-1', 'Runbook', 'preserved document', ?1, ?2, ?2)",
            rusqlite::params![project_id, now],
        )
        .expect("document");
        conn.execute(
            "INSERT INTO repository (id, workspace_id, name, path, provider, remote_url, project_id, \
             created_at, updated_at) VALUES ('repo-1', 'ws-1', 'ForgeDesk', \
             '/tmp/forgedesk-v5-repo', 'local', 'git@github.com:mindwork64/forgedesk.git', ?1, ?2, ?2)",
            rusqlite::params![project_id, now],
        )
        .expect("repository");
        conn.execute(
            "INSERT INTO repository (id, workspace_id, name, path, provider, remote_url, \
             created_at, updated_at) VALUES ('repo-2', 'ws-1', 'Other', '/tmp/forgedesk-other', \
             'local', 'https://github.com/mindwork64/other.git', ?1, ?1)",
            [now],
        )
        .expect("https repository");
        conn.execute(
            "INSERT INTO project (id, workspace_id, name, description, status, priority, \
             created_at, updated_at) VALUES ('project-2', 'ws-1', 'Release clone', \
             'separate project', 'active', 'medium', ?1, ?1)",
            [now],
        )
        .expect("second project");
        conn.execute(
            "INSERT INTO repository (id, workspace_id, name, path, provider, remote_url, project_id, \
             created_at, updated_at) VALUES ('repo-3', 'ws-1', 'ForgeDesk clone 2', \
             '/tmp/forgedesk-v5-repo-2', 'local', 'https://github.com/mindwork64/forgedesk.git', \
             'project-2', ?1, ?1)",
            [now],
        )
        .expect("duplicate origin checkout");
        conn.execute(
            "INSERT INTO remote_issue_link (id, local_issue_id, provider, account_id, \
             repository_id, remote_id, remote_number, remote_url, created_at, updated_at) \
             VALUES ('link-1', ?1, 'github', 'account-1', 'owner/repo', 'R-42', 42, \
             'https://github.com/owner/repo/issues/42', ?2, ?2)",
            rusqlite::params![issue_id, now],
        )
        .expect("remote link");
        conn.execute(
            "INSERT INTO sync_outbox (id, local_issue_id, operation, payload_json, state, \
             attempts, idempotency_key, fingerprint, created_at, updated_at) \
             VALUES ('outbox-1', ?1, 'create_issue', '{\"title\":\"FD-1\"}', 'pending', 0, \
             'idem-1', 'fingerprint-1', ?2, ?2)",
            rusqlite::params![issue_id, now],
        )
        .expect("outbox");
        conn.execute(
            "INSERT INTO sync_cursor (id, provider, account_id, resource, scope, \
             page_cursor, watermark, last_success_at, updated_at) \
             VALUES ('cursor-1', 'github', 'account-1', 'issues', 'owner/repo', '2', ?1, ?1, ?1)",
            [now],
        )
        .expect("cursor");

        assert_eq!(run(&mut conn).expect("upgrade"), CURRENT_SCHEMA_VERSION);
        assert_eq!(
            current_schema_version(&conn).expect("version"),
            CURRENT_SCHEMA_VERSION
        );

        assert_eq!(
            conn.query_row(
                "SELECT name FROM project WHERE id = 'project-1'",
                [],
                |row| { row.get::<_, String>(0) }
            )
            .expect("project"),
            "Release"
        );
        let issue: (String, String, String) = conn
            .query_row(
                "SELECT title, status_id, project_id FROM issue WHERE id = 'issue-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("issue");
        assert_eq!(issue.0, "Preserved issue");
        assert_eq!(issue.1, todo_status);
        assert_eq!(issue.2, project_id);
        assert_eq!(
            conn.query_row(
                "SELECT body FROM comment WHERE id = 'comment-1'",
                [],
                |row| { row.get::<_, String>(0) }
            )
            .expect("comment"),
            "preserved comment"
        );
        assert_eq!(
            conn.query_row("SELECT body FROM document WHERE id = 'doc-1'", [], |row| {
                row.get::<_, String>(0)
            })
            .expect("document"),
            "preserved document"
        );
        assert_eq!(
            conn.query_row(
                "SELECT path FROM repository_checkout WHERE repository_id = 'repo-1'",
                [],
                |row| { row.get::<_, String>(0) }
            )
            .expect("repository"),
            "/tmp/forgedesk-v5-repo"
        );
        assert_eq!(
            conn.query_row(
                "SELECT provider || '|' || host || '|' || full_name || '|' || canonical_remote_url \
                 FROM repository WHERE id = 'repo-2'",
                [],
                |row| row.get::<_, String>(0)
            )
            .expect("https normalized"),
            "github|github.com|mindwork64/other|https://github.com/mindwork64/other"
        );
        assert_eq!(
            conn.query_row(
                "SELECT provider || '|' || host || '|' || full_name || '|' || \
                 COALESCE(canonical_remote_url, '<none>') || '|' || \
                 CASE WHEN identity_conflict IS NULL THEN 'clear' ELSE 'conflict' END \
                 FROM repository WHERE id = 'repo-3'",
                [],
                |row| row.get::<_, String>(0)
            )
            .expect("duplicate identity"),
            "local|local|ForgeDesk clone 2|<none>|conflict"
        );
        assert_eq!(
            conn.query_row(
                "SELECT path FROM repository_checkout WHERE repository_id = 'repo-3'",
                [],
                |row| row.get::<_, String>(0)
            )
            .expect("conflict checkout"),
            "/tmp/forgedesk-v5-repo-2"
        );
        assert_eq!(
            conn.query_row(
                "SELECT count(*) FROM repository_project \
                 WHERE repository_id = 'repo-3' AND project_id = 'project-2'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .expect("conflict project link"),
            1
        );
        let discovered = crate::db::repositories::repo_registry::upsert_remote(
            &conn,
            "ws-1",
            &crate::db::repositories::repo_registry::RemoteRepositoryInput {
                provider: "github".into(),
                host: "github.com".into(),
                external_id: Some("123".into()),
                owner: Some("mindwork64".into()),
                name: "forgedesk".into(),
                full_name: "mindwork64/forgedesk".into(),
                description: None,
                visibility: None,
                default_branch: Some("main".into()),
                html_url: Some("https://github.com/mindwork64/forgedesk".into()),
                clone_url: Some("https://github.com/mindwork64/forgedesk.git".into()),
                canonical_remote_url: Some("https://github.com/mindwork64/forgedesk".into()),
                is_archived: false,
            },
        )
        .expect("discovery reconcile");
        assert_eq!(discovered.id, "repo-1");
        assert_eq!(discovered.checkouts.len(), 1);
        assert_eq!(discovered.projects.len(), 1);

        assert_eq!(
            conn.query_row(
                "SELECT count(*) FROM repository_project                  WHERE workspace_id = 'ws-1' AND repository_id = 'repo-1'                    AND project_id = 'project-1'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .expect("project link"),
            1
        );
        assert_eq!(
            conn.query_row(
                "SELECT remote_repository_key FROM remote_issue_link WHERE id = 'link-1'",
                [],
                |row| row.get::<_, String>(0)
            )
            .expect("remote repository key"),
            "owner/repo"
        );
        assert_eq!(
            conn.query_row(
                "SELECT remote_number FROM remote_issue_link WHERE id = 'link-1'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .expect("link"),
            42
        );
        assert_eq!(
            conn.query_row(
                "SELECT fingerprint FROM sync_outbox WHERE id = 'outbox-1'",
                [],
                |row| row.get::<_, String>(0)
            )
            .expect("outbox"),
            "fingerprint-1"
        );
        assert_eq!(
            conn.query_row(
                "SELECT watermark FROM sync_cursor WHERE id = 'cursor-1'",
                [],
                |row| { row.get::<_, String>(0) }
            )
            .expect("cursor"),
            now
        );
        assert_eq!(
            conn.query_row(
                "SELECT value FROM app_setting WHERE key = 'theme'",
                [],
                |row| { row.get::<_, String>(0) }
            )
            .expect("setting"),
            "\"dark\""
        );

        let linked: i64 = conn
            .query_row(
                "SELECT count(*) FROM issue i                  JOIN project p ON p.id = i.project_id                  JOIN workflow_status s ON s.id = i.status_id                  JOIN comment c ON c.issue_id = i.id                  JOIN remote_issue_link l ON l.local_issue_id = i.id                  JOIN sync_outbox o ON o.local_issue_id = i.id                  WHERE i.id = 'issue-1' AND p.id = 'project-1'                    AND s.system_key = 'todo' AND c.id = 'comment-1'                    AND l.id = 'link-1' AND o.id = 'outbox-1'",
                [],
                |row| row.get(0),
            )
            .expect("relationships");
        assert_eq!(linked, 1);

        for table in [
            "pull_request_link",
            "inbox_state",
            "automation_rule",
            "automation_run",
            "dashboard",
            "dashboard_widget",
        ] {
            let exists: i64 = conn
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table],
                    |row| row.get(0),
                )
                .expect("table check");
            assert_eq!(exists, 1, "missing {table}");
        }

        let mut check = conn.prepare("PRAGMA foreign_key_check").expect("fk check");
        let violations = check.query_map([], |_| Ok(())).expect("rows").count();
        assert_eq!(violations, 0, "foreign key violations after migration");
    }

    #[test]
    fn reverse_insertion_keeps_primary_identity_and_preserves_both_links() {
        let mut conn = connection();
        apply_through(&mut conn, 1);
        conn.execute(
            "INSERT INTO workspace (id, name, key_prefix, created_at, updated_at) \
             VALUES ('ws-1', 'Reverse', 'RV', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        )
        .expect("workspace");
        apply_through(&mut conn, 9);
        for (id, name) in [("project-a", "A"), ("project-b", "B")] {
            conn.execute(
                "INSERT INTO project (id, workspace_id, name, description, status, priority, \
                 created_at, updated_at) VALUES (?1, 'ws-1', ?2, '', 'active', 'none', \
                 '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                rusqlite::params![id, name],
            )
            .expect("project");
        }
        // Insert the higher ID first: deterministic reconciliation must still choose legacy-a.
        for (id, project, path) in [
            ("legacy-b", "project-b", "/tmp/checkout-b"),
            ("legacy-a", "project-a", "/tmp/checkout-a"),
        ] {
            conn.execute(
                "INSERT INTO repository (id, workspace_id, name, path, provider, remote_url, \
                 project_id, created_at, updated_at) VALUES (?1, 'ws-1', ?1, ?2, 'local', \
                 'https://github.com/owner/repo.git', ?3, '2026-01-01T00:00:00Z', \
                 '2026-01-01T00:00:00Z')",
                rusqlite::params![id, path, project],
            )
            .expect("legacy repository");
        }

        assert_eq!(run(&mut conn).expect("migrate"), CURRENT_SCHEMA_VERSION);
        assert_eq!(
            conn.query_row(
                "SELECT id FROM repository WHERE canonical_remote_url IS NOT NULL",
                [],
                |row| row.get::<_, String>(0)
            )
            .expect("primary"),
            "legacy-a"
        );
        assert_eq!(
            conn.query_row(
                "SELECT id FROM repository WHERE identity_conflict IS NOT NULL",
                [],
                |row| row.get::<_, String>(0)
            )
            .expect("conflict"),
            "legacy-b"
        );
        for (repository, project, path) in [
            ("legacy-a", "project-a", "/tmp/checkout-a"),
            ("legacy-b", "project-b", "/tmp/checkout-b"),
        ] {
            assert_eq!(
                conn.query_row(
                    "SELECT path FROM repository_checkout WHERE repository_id = ?1",
                    [repository],
                    |row| row.get::<_, String>(0)
                )
                .expect("checkout"),
                path
            );
            assert_eq!(
                conn.query_row(
                    "SELECT count(*) FROM repository_project \
                     WHERE repository_id = ?1 AND project_id = ?2",
                    rusqlite::params![repository, project],
                    |row| row.get::<_, i64>(0)
                )
                .expect("project link"),
                1
            );
        }
        let mut check = conn.prepare("PRAGMA foreign_key_check").expect("fk check");
        assert_eq!(check.query_map([], |_| Ok(())).expect("rows").count(), 0);
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

        assert_eq!(
            run(&mut conn).expect("migrate to current"),
            CURRENT_SCHEMA_VERSION
        );

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
