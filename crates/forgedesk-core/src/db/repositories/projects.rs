use rusqlite::{Connection, OptionalExtension};

use crate::domain::time::now_rfc3339;
use crate::domain::ProjectRef;
use crate::error::{CoreError, CoreResult};

pub const NAME_MAX: usize = 120;

pub fn list(conn: &Connection, workspace_id: &str) -> CoreResult<Vec<ProjectRef>> {
    let mut statement = conn.prepare(
        "SELECT id, name FROM project WHERE workspace_id = ?1 AND archived_at IS NULL \
         ORDER BY name COLLATE NOCASE ASC",
    )?;
    let rows = statement.query_map([workspace_id], |row| {
        Ok(ProjectRef {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn find(
    conn: &Connection,
    workspace_id: &str,
    project_id: &str,
) -> CoreResult<Option<ProjectRef>> {
    Ok(conn
        .query_row(
            "SELECT id, name FROM project \
             WHERE workspace_id = ?1 AND id = ?2 AND archived_at IS NULL",
            rusqlite::params![workspace_id, project_id],
            |row| {
                Ok(ProjectRef {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            },
        )
        .optional()?)
}

/// Minimal local project creation used by Stage 3 and issue tests. The Projects
/// screen and full project lifecycle are a later stage.
pub fn create(conn: &Connection, workspace_id: &str, name: &str) -> CoreResult<ProjectRef> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > NAME_MAX {
        return Err(CoreError::validation(
            "name",
            format!("Project name must be between 1 and {NAME_MAX} characters."),
        ));
    }
    let project = ProjectRef {
        id: uuid::Uuid::now_v7().to_string(),
        name: name.to_owned(),
    };
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO project (id, workspace_id, name, description, status, priority, created_at, updated_at) \
         VALUES (?1, ?2, ?3, '', 'planned', 'none', ?4, ?4)",
        rusqlite::params![project.id, workspace_id, project.name, now],
    )?;
    Ok(project)
}
