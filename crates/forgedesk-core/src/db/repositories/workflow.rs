use rusqlite::{Connection, OptionalExtension};

use crate::domain::time::now_rfc3339;
use crate::domain::{StatusCategory, StatusColor, StatusSummary, Workflow, WorkflowStatus};
use crate::error::{CoreError, CoreResult};

const STATUS_ORDER: &str = "s.position ASC, s.name ASC";

struct DefaultStatus {
    key: &'static str,
    name: &'static str,
    category: StatusCategory,
    color: StatusColor,
    is_default: bool,
}

const DEFAULT_STATUSES: &[DefaultStatus] = &[
    DefaultStatus {
        key: "backlog",
        name: "Backlog",
        category: StatusCategory::Backlog,
        color: StatusColor::Neutral,
        is_default: true,
    },
    DefaultStatus {
        key: "todo",
        name: "Todo",
        category: StatusCategory::Unstarted,
        color: StatusColor::Info,
        is_default: false,
    },
    DefaultStatus {
        key: "in-progress",
        name: "In Progress",
        category: StatusCategory::Started,
        color: StatusColor::Accent,
        is_default: false,
    },
    DefaultStatus {
        key: "done",
        name: "Done",
        category: StatusCategory::Completed,
        color: StatusColor::Success,
        is_default: false,
    },
    DefaultStatus {
        key: "canceled",
        name: "Canceled",
        category: StatusCategory::Canceled,
        color: StatusColor::Neutral,
        is_default: false,
    },
];

pub fn parse_status(
    id: String,
    name: String,
    category: &str,
    color: &str,
    position: i64,
    system_key: Option<String>,
) -> CoreResult<StatusSummary> {
    Ok(StatusSummary {
        id,
        name,
        category: StatusCategory::parse(category)?,
        color: StatusColor::parse(color)?,
        position,
        system_key,
    })
}

pub fn load(conn: &Connection, workspace_id: &str) -> CoreResult<Option<Workflow>> {
    let workflow = conn
        .query_row(
            "SELECT id, name, is_default FROM workflow WHERE workspace_id = ?1 \
             ORDER BY is_default DESC, created_at ASC LIMIT 1",
            [workspace_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)? != 0,
                ))
            },
        )
        .optional()?;

    let Some((id, name, is_default)) = workflow else {
        return Ok(None);
    };

    let mut statement = conn.prepare(&format!(
        "SELECT s.id, s.name, s.category, s.color, s.position, s.system_key, s.is_default \
         FROM workflow_status s WHERE s.workflow_id = ?1 ORDER BY {STATUS_ORDER}"
    ))?;
    let rows = statement.query_map([&id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, i64>(6)? != 0,
        ))
    })?;

    let mut statuses = Vec::new();
    for row in rows {
        let (status_id, status_name, category, color, position, system_key, default) = row?;
        statuses.push(WorkflowStatus {
            summary: parse_status(
                status_id,
                status_name,
                &category,
                &color,
                position,
                system_key,
            )?,
            is_default: default,
        });
    }

    Ok(Some(Workflow {
        id,
        name,
        is_default,
        statuses,
    }))
}

/// Seed the single default workflow for a newly created workspace.
pub fn seed_defaults(conn: &Connection, workspace_id: &str) -> CoreResult<Workflow> {
    if let Some(workflow) = load(conn, workspace_id)? {
        return Ok(workflow);
    }

    let now = now_rfc3339();
    let workflow_id = format!("wf-{workspace_id}");
    conn.execute(
        "INSERT INTO workflow (id, workspace_id, name, is_default, created_at) \
         VALUES (?1, ?2, 'Default', 1, ?3)",
        rusqlite::params![workflow_id, workspace_id, now],
    )?;

    for (position, status) in DEFAULT_STATUSES.iter().enumerate() {
        let status_id = format!("st-{workspace_id}-{}", status.key);
        conn.execute(
            "INSERT INTO workflow_status \
             (id, workflow_id, name, category, color, position, is_default, system_key, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                status_id,
                workflow_id,
                status.name,
                status.category.as_str(),
                status.color.as_str(),
                position as i64,
                i64::from(status.is_default),
                status.key,
                now,
            ],
        )?;
    }

    load(conn, workspace_id)?
        .ok_or_else(|| CoreError::internal("Workflow seed did not produce a workflow."))
}

pub fn statuses(conn: &Connection, workspace_id: &str) -> CoreResult<Vec<StatusSummary>> {
    Ok(load(conn, workspace_id)?
        .map(|workflow| {
            workflow
                .statuses
                .into_iter()
                .map(|status| status.summary)
                .collect()
        })
        .unwrap_or_default())
}

pub fn find_status(
    conn: &Connection,
    workspace_id: &str,
    status_id: &str,
) -> CoreResult<Option<StatusSummary>> {
    conn.query_row(
        "SELECT s.id, s.name, s.category, s.color, s.position, s.system_key \
         FROM workflow_status s JOIN workflow w ON w.id = s.workflow_id \
         WHERE w.workspace_id = ?1 AND s.id = ?2",
        rusqlite::params![workspace_id, status_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        },
    )
    .optional()?
    .map(|(id, name, category, color, position, system_key)| {
        parse_status(id, name, &category, &color, position, system_key)
    })
    .transpose()
}

pub fn default_status(conn: &Connection, workspace_id: &str) -> CoreResult<StatusSummary> {
    let row = conn
        .query_row(
            &format!(
                "SELECT s.id, s.name, s.category, s.color, s.position, s.system_key \
                 FROM workflow_status s JOIN workflow w ON w.id = s.workflow_id \
                 WHERE w.workspace_id = ?1 \
                 ORDER BY s.is_default DESC, {STATUS_ORDER} LIMIT 1"
            ),
            [workspace_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            },
        )
        .optional()?;

    let Some((id, name, category, color, position, system_key)) = row else {
        return Err(CoreError::internal(
            "The workspace has no workflow status. Reopen the workspace to repair it.",
        ));
    };
    parse_status(id, name, &category, &color, position, system_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn seeds_and_loads_default_workflow() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("open");
        let workspace = db
            .with_transaction(|tx| {
                let workspace =
                    crate::db::repositories::workspace::create_initial(tx, "Workflow Test")?;
                Ok(workspace)
            })
            .expect("workspace");

        let loaded = db
            .with_conn(|conn| load(conn, &workspace.id))
            .expect("load")
            .expect("workflow");
        assert_eq!(loaded.statuses.len(), 5);
        assert_eq!(loaded.statuses[0].summary.category, StatusCategory::Backlog);
        assert!(loaded.statuses[0].is_default);
        assert_eq!(loaded.statuses[2].summary.name, "In Progress");

        let default = db
            .with_conn(|conn| default_status(conn, &workspace.id))
            .expect("default");
        assert_eq!(default.name, "Backlog");
    }
}
