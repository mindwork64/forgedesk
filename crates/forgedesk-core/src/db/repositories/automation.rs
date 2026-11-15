use rusqlite::{Connection, OptionalExtension, Row};

use crate::domain::automation::{AutomationEvent, AutomationInput, AutomationRule, AutomationRun};
use crate::domain::time::now_rfc3339;
use crate::error::{CoreError, CoreResult};

pub fn list_rules(conn: &Connection, workspace_id: &str) -> CoreResult<Vec<AutomationRule>> {
    let mut statement = conn.prepare(
        "SELECT r.id, r.name, r.enabled, r.trigger_event, r.action_type, r.action_value, \
                r.last_run_at, r.last_result, \
                (SELECT count(*) FROM automation_run x WHERE x.rule_id = r.id), \
                r.created_at, r.updated_at \
         FROM automation_rule r WHERE r.workspace_id = ?1 ORDER BY r.created_at ASC",
    )?;
    let rows = statement.query_map([workspace_id], read_rule)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn create_rule(
    conn: &Connection,
    workspace_id: &str,
    input: &AutomationInput,
) -> CoreResult<AutomationRule> {
    input.validate()?;
    let id = uuid::Uuid::now_v7().to_string();
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO automation_rule \
         (id, workspace_id, name, enabled, trigger_event, action_type, action_value, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, 'add_label', ?6, ?7, ?7)",
        rusqlite::params![
            id,
            workspace_id,
            input.name.trim(),
            i64::from(input.enabled),
            input.trigger_event,
            input.action_value.trim(),
            now
        ],
    )?;
    get_rule(conn, workspace_id, &id)
}

pub fn set_enabled(
    conn: &Connection,
    workspace_id: &str,
    rule_id: &str,
    enabled: bool,
) -> CoreResult<AutomationRule> {
    let changed = conn.execute(
        "UPDATE automation_rule SET enabled = ?1, updated_at = ?2 WHERE id = ?3 AND workspace_id = ?4",
        rusqlite::params![i64::from(enabled), now_rfc3339(), rule_id, workspace_id],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Automation"));
    }
    get_rule(conn, workspace_id, rule_id)
}

pub fn delete_rule(conn: &Connection, workspace_id: &str, rule_id: &str) -> CoreResult<()> {
    let changed = conn.execute(
        "DELETE FROM automation_rule WHERE id = ?1 AND workspace_id = ?2",
        rusqlite::params![rule_id, workspace_id],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Automation"));
    }
    Ok(())
}

pub fn preview(
    conn: &Connection,
    workspace_id: &str,
    rule_id: &str,
) -> CoreResult<Vec<AutomationEvent>> {
    let rule = get_rule(conn, workspace_id, rule_id)?;
    let mut statement = conn.prepare(
        "SELECT a.id, i.id, i.key, i.title, a.created_at \
         FROM activity_event a JOIN issue i ON i.id = a.issue_id \
         WHERE i.workspace_id = ?1 AND a.kind = ?2 AND i.deleted_at IS NULL \
         ORDER BY a.created_at DESC, a.rowid DESC LIMIT 20",
    )?;
    let rows = statement.query_map(rusqlite::params![workspace_id, rule.trigger_event], |row| {
        Ok(AutomationEvent {
            event_id: row.get(0)?,
            entity_id: row.get(1)?,
            entity_key: row.get(2)?,
            entity_title: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn run_rule(conn: &Connection, workspace_id: &str, rule_id: &str) -> CoreResult<i64> {
    let rule = get_rule(conn, workspace_id, rule_id)?;
    if !rule.enabled {
        return Err(CoreError::validation("enabled", "Automation is disabled."));
    }
    let mut statement = conn.prepare(
        "SELECT a.id, a.issue_id FROM activity_event a JOIN issue i ON i.id = a.issue_id \
         WHERE i.workspace_id = ?1 AND a.kind = ?2 AND i.deleted_at IS NULL \
           AND NOT EXISTS (SELECT 1 FROM automation_run r WHERE r.rule_id = ?3 AND r.event_id = a.id) \
         ORDER BY a.created_at ASC, a.rowid ASC LIMIT 20",
    )?;
    let rows = statement.query_map(
        rusqlite::params![workspace_id, rule.trigger_event, rule_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    )?;
    let events = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    let now = now_rfc3339();
    for (event_id, issue_id) in &events {
        crate::db::repositories::labels::add_to_issue(
            conn,
            workspace_id,
            issue_id,
            std::slice::from_ref(&rule.action_value),
        )?;
        conn.execute(
            "INSERT INTO automation_run (id, rule_id, event_id, status, message, created_at) \
             VALUES (?1, ?2, ?3, 'success', ?4, ?5)",
            rusqlite::params![
                uuid::Uuid::now_v7().to_string(),
                rule_id,
                event_id,
                format!("Added label {}", rule.action_value),
                now
            ],
        )?;
    }
    let message = if events.is_empty() {
        "No new matching events".to_owned()
    } else {
        format!("Applied to {} event(s)", events.len())
    };
    conn.execute(
        "UPDATE automation_rule SET last_run_at = ?1, last_result = ?2, updated_at = ?1 \
         WHERE id = ?3",
        rusqlite::params![now, message, rule_id],
    )?;
    Ok(events.len() as i64)
}

pub fn runs(
    conn: &Connection,
    workspace_id: &str,
    rule_id: &str,
) -> CoreResult<Vec<AutomationRun>> {
    get_rule(conn, workspace_id, rule_id)?;
    let mut statement = conn.prepare(
        "SELECT id, rule_id, event_id, status, message, created_at FROM automation_run \
         WHERE rule_id = ?1 ORDER BY created_at DESC LIMIT 100",
    )?;
    let rows = statement.query_map([rule_id], |row| {
        Ok(AutomationRun {
            id: row.get(0)?,
            rule_id: row.get(1)?,
            event_id: row.get(2)?,
            status: row.get(3)?,
            message: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_rule(
    conn: &Connection,
    workspace_id: &str,
    rule_id: &str,
) -> CoreResult<AutomationRule> {
    conn.query_row(
        "SELECT r.id, r.name, r.enabled, r.trigger_event, r.action_type, r.action_value, \
                r.last_run_at, r.last_result, \
                (SELECT count(*) FROM automation_run x WHERE x.rule_id = r.id), \
                r.created_at, r.updated_at \
         FROM automation_rule r WHERE r.id = ?1 AND r.workspace_id = ?2",
        rusqlite::params![rule_id, workspace_id],
        read_rule,
    )
    .optional()?
    .ok_or_else(|| CoreError::not_found("Automation"))
}

fn read_rule(row: &Row<'_>) -> rusqlite::Result<AutomationRule> {
    Ok(AutomationRule {
        id: row.get(0)?,
        name: row.get(1)?,
        enabled: row.get::<_, i64>(2)? != 0,
        trigger_event: row.get(3)?,
        action_type: row.get(4)?,
        action_value: row.get(5)?,
        last_run_at: row.get(6)?,
        last_result: row.get(7)?,
        run_count: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
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
            .with_conn(|conn| {
                crate::db::repositories::workspace::create_initial(conn, "Automation")
            })
            .expect("workspace");
        let issue = db
            .with_conn(|conn| {
                crate::db::repositories::issues::create(
                    conn,
                    &workspace.id,
                    &CreateIssueInput {
                        title: "Automation issue".into(),
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
        (dir, db, workspace.id, issue)
    }

    #[test]
    fn runs_once_per_event_and_is_idempotent() {
        let (_dir, db, workspace_id, issue_id) = fixture();
        let rule = db
            .with_conn(|conn| {
                create_rule(
                    conn,
                    &workspace_id,
                    &AutomationInput {
                        name: "Label new issues".into(),
                        trigger_event: "issue_created".into(),
                        action_value: "automated".into(),
                        enabled: true,
                    },
                )
            })
            .expect("rule");
        let preview = db
            .with_conn(|conn| preview(conn, &workspace_id, &rule.id))
            .expect("preview");
        assert_eq!(preview.len(), 1);
        let first = db
            .with_conn(|conn| run_rule(conn, &workspace_id, &rule.id))
            .expect("run");
        assert_eq!(first, 1);
        let second = db
            .with_conn(|conn| run_rule(conn, &workspace_id, &rule.id))
            .expect("repeat");
        assert_eq!(second, 0);
        let history = db
            .with_conn(|conn| runs(conn, &workspace_id, &rule.id))
            .expect("runs");
        assert_eq!(history.len(), 1);
        let detail = db
            .with_conn(|conn| {
                crate::db::repositories::issues::get_detail(conn, &workspace_id, &issue_id)
            })
            .expect("detail");
        assert!(detail
            .summary
            .labels
            .iter()
            .any(|label| label.name == "automated"));
    }

    #[test]
    fn disabled_rule_cannot_run() {
        let (_dir, db, workspace_id, _issue_id) = fixture();
        let rule = db
            .with_conn(|conn| {
                create_rule(
                    conn,
                    &workspace_id,
                    &AutomationInput {
                        name: "Disabled".into(),
                        trigger_event: "issue_created".into(),
                        action_value: "x".into(),
                        enabled: false,
                    },
                )
            })
            .expect("rule");
        assert!(db
            .with_conn(|conn| run_rule(conn, &workspace_id, &rule.id))
            .is_err());
    }
}
