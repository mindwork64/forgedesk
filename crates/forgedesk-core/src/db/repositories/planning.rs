use chrono::Utc;
use rusqlite::types::Value as SqlValue;
use rusqlite::{params_from_iter, Connection, OptionalExtension, Row};

use crate::domain::issue::{Priority, StatusSummary};
use crate::domain::planning::{
    markdown_export, validate_body, validate_cycle_dates, validate_date_range,
    validate_document_title, validate_name, Cycle, CycleDetail, CycleInput, CycleStatus,
    CycleSummary, Document, DocumentInput, Initiative, InitiativeDetail, InitiativeInput,
    InitiativeRef, Milestone, MilestoneInput, MilestoneStatus, PlanningStatus, Progress, Project,
    ProjectDetail, ProjectInput, ProjectQuery, StatusCount,
};
use crate::domain::time::now_rfc3339;
use crate::domain::workspace::Workspace;
use crate::error::{CoreError, CoreResult};

use super::issues;

const PROJECT_SELECT: &str = "\
SELECT p.id, p.name, p.description, p.status, p.priority, p.start_date, p.target_date, \
       p.color, p.archived_at, p.created_at, p.updated_at, i.id, i.name, \
       (SELECT count(*) FROM issue x WHERE x.project_id = p.id AND x.deleted_at IS NULL AND x.archived_at IS NULL), \
       (SELECT count(*) FROM issue x JOIN workflow_status s ON s.id = x.status_id \
          WHERE x.project_id = p.id AND x.deleted_at IS NULL AND x.archived_at IS NULL AND s.category = 'completed'), \
       (SELECT count(*) FROM issue x JOIN workflow_status s ON s.id = x.status_id \
          WHERE x.project_id = p.id AND x.deleted_at IS NULL AND x.archived_at IS NULL AND s.category = 'canceled'), \
       p.issue_key_prefix \
FROM project p LEFT JOIN initiative i ON i.id = p.initiative_id";

const MILESTONE_SELECT: &str = "\
SELECT m.id, m.project_id, m.name, m.description, m.due_date, m.status, \
       m.created_at, m.updated_at, \
       (SELECT count(*) FROM issue_milestone im JOIN issue x ON x.id = im.issue_id \
          WHERE im.milestone_id = m.id AND x.deleted_at IS NULL AND x.archived_at IS NULL), \
       (SELECT count(*) FROM issue_milestone im JOIN issue x ON x.id = im.issue_id \
          JOIN workflow_status s ON s.id = x.status_id \
          WHERE im.milestone_id = m.id AND x.deleted_at IS NULL AND x.archived_at IS NULL \
            AND s.category = 'completed'), \
       (SELECT count(*) FROM issue_milestone im JOIN issue x ON x.id = im.issue_id \
          JOIN workflow_status s ON s.id = x.status_id \
          WHERE im.milestone_id = m.id AND x.deleted_at IS NULL AND x.archived_at IS NULL \
            AND s.category = 'canceled') \
FROM milestone m";

pub fn list_projects(
    conn: &Connection,
    workspace_id: &str,
    query: &ProjectQuery,
) -> CoreResult<Vec<Project>> {
    let mut sql = format!("{PROJECT_SELECT} WHERE p.workspace_id = ?");
    let mut params = vec![SqlValue::Text(workspace_id.to_owned())];
    if !query.include_archived {
        sql.push_str(" AND p.archived_at IS NULL");
    }
    if let Some(status) = query.status {
        sql.push_str(" AND p.status = ?");
        params.push(SqlValue::Text(status.as_str().to_owned()));
    }
    if let Some(initiative_id) = &query.initiative_id {
        sql.push_str(" AND p.initiative_id = ?");
        params.push(SqlValue::Text(initiative_id.clone()));
    }
    if let Some(search) = query
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        sql.push_str(" AND (lower(p.name) LIKE ? OR lower(p.description) LIKE ?)");
        let pattern = format!("%{}%", search.to_lowercase());
        params.push(SqlValue::Text(pattern.clone()));
        params.push(SqlValue::Text(pattern));
    }
    sql.push_str(" ORDER BY p.updated_at DESC, p.name COLLATE NOCASE ASC");

    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(params.iter()), read_project)?;
    rows.map(|row| row.map_err(CoreError::from))
        .collect::<CoreResult<Vec<_>>>()
}

pub fn get_project(conn: &Connection, workspace_id: &str, project_id: &str) -> CoreResult<Project> {
    conn.query_row(
        &format!("{PROJECT_SELECT} WHERE p.workspace_id = ?1 AND p.id = ?2"),
        rusqlite::params![workspace_id, project_id],
        read_project,
    )
    .optional()?
    .ok_or_else(|| CoreError::not_found("Project"))
}

pub fn project_detail(
    conn: &Connection,
    workspace_id: &str,
    project_id: &str,
) -> CoreResult<ProjectDetail> {
    let project = get_project(conn, workspace_id, project_id)?;
    let milestones = list_milestones(conn, project_id)?;
    let mut statement = conn.prepare(
        "SELECT s.id, s.name, s.category, s.color, s.position, s.system_key, count(x.id) \
         FROM issue x JOIN workflow_status s ON s.id = x.status_id \
         WHERE x.project_id = ?1 AND x.deleted_at IS NULL AND x.archived_at IS NULL \
         GROUP BY s.id ORDER BY s.position ASC",
    )?;
    let rows = statement.query_map([project_id], |row| {
        Ok(StatusCount {
            status: read_status(row, 0)?,
            count: row.get(6)?,
        })
    })?;
    let issue_counts = rows.collect::<Result<Vec<_>, _>>()?;
    let issue_count = issue_counts.iter().map(|count| count.count).sum();
    Ok(ProjectDetail {
        project,
        milestones,
        issue_counts,
        issue_count,
    })
}

pub fn create_project(
    conn: &Connection,
    workspace_id: &str,
    input: &ProjectInput,
) -> CoreResult<Project> {
    let name = validate_name("name", &input.name)?;
    let issue_key_prefix = normalize_project_key_prefix(input.issue_key_prefix.as_deref())?;
    validate_date_range(input.start_date.as_deref(), input.target_date.as_deref())?;
    if let Some(initiative_id) = &input.initiative_id {
        ensure_initiative(conn, workspace_id, initiative_id)?;
    }
    let id = uuid::Uuid::now_v7().to_string();
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO project \
         (id, workspace_id, name, description, status, priority, start_date, target_date, \
          initiative_id, color, issue_key_prefix, progress_method, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'issues', ?12, ?12)",
        rusqlite::params![
            id,
            workspace_id,
            name,
            input.description,
            input.status.unwrap_or(PlanningStatus::Planned).as_str(),
            input.priority.unwrap_or(Priority::None).as_str(),
            input.start_date,
            input.target_date,
            input.initiative_id,
            input.color.clone().unwrap_or_else(|| "accent".into()),
            issue_key_prefix,
            now,
        ],
    )?;
    get_project(conn, workspace_id, &id)
}

pub fn update_project(
    conn: &Connection,
    workspace_id: &str,
    project_id: &str,
    input: &ProjectInput,
) -> CoreResult<Project> {
    let name = validate_name("name", &input.name)?;
    let issue_key_prefix = normalize_project_key_prefix(input.issue_key_prefix.as_deref())?;
    validate_date_range(input.start_date.as_deref(), input.target_date.as_deref())?;
    if let Some(initiative_id) = &input.initiative_id {
        ensure_initiative(conn, workspace_id, initiative_id)?;
    }
    let changed = conn.execute(
        "UPDATE project SET name = ?1, description = ?2, status = ?3, priority = ?4, \
         start_date = ?5, target_date = ?6, initiative_id = ?7, color = ?8, \
         issue_key_prefix = ?9, updated_at = ?10 \
         WHERE workspace_id = ?11 AND id = ?12",
        rusqlite::params![
            name,
            input.description,
            input.status.unwrap_or(PlanningStatus::Planned).as_str(),
            input.priority.unwrap_or(Priority::None).as_str(),
            input.start_date,
            input.target_date,
            input.initiative_id,
            input.color.clone().unwrap_or_else(|| "accent".into()),
            issue_key_prefix,
            now_rfc3339(),
            workspace_id,
            project_id,
        ],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Project"));
    }
    get_project(conn, workspace_id, project_id)
}

pub fn archive_project(
    conn: &Connection,
    workspace_id: &str,
    project_id: &str,
    archived: bool,
) -> CoreResult<Project> {
    let now = now_rfc3339();
    let changed = conn.execute(
        "UPDATE project SET archived_at = ?1, updated_at = ?2 \
         WHERE workspace_id = ?3 AND id = ?4",
        rusqlite::params![
            archived.then_some(now.clone()),
            now,
            workspace_id,
            project_id
        ],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Project"));
    }
    get_project(conn, workspace_id, project_id)
}

pub fn list_milestones(conn: &Connection, project_id: &str) -> CoreResult<Vec<Milestone>> {
    let mut statement = conn.prepare(&format!(
        "{MILESTONE_SELECT} WHERE m.project_id = ?1 AND m.archived_at IS NULL \
         ORDER BY m.due_date IS NULL, m.due_date ASC, m.created_at ASC"
    ))?;
    let rows = statement.query_map([project_id], read_milestone)?;
    rows.map(|row| row.map_err(CoreError::from))
        .collect::<CoreResult<Vec<_>>>()
}

pub fn create_milestone(
    conn: &Connection,
    workspace_id: &str,
    project_id: &str,
    input: &MilestoneInput,
) -> CoreResult<Milestone> {
    get_project(conn, workspace_id, project_id)?;
    let name = validate_name("name", &input.name)?;
    validate_date_range(None, input.due_date.as_deref())?;
    let id = uuid::Uuid::now_v7().to_string();
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO milestone \
         (id, project_id, name, description, due_date, status, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        rusqlite::params![
            id,
            project_id,
            name,
            input.description,
            input.due_date,
            input.status.unwrap_or(MilestoneStatus::Planned).as_str(),
            now,
        ],
    )?;
    get_milestone(conn, &id)
}

pub fn update_milestone(
    conn: &Connection,
    workspace_id: &str,
    milestone_id: &str,
    input: &MilestoneInput,
) -> CoreResult<Milestone> {
    let project_id: String = conn
        .query_row(
            "SELECT m.project_id FROM milestone m JOIN project p ON p.id = m.project_id \
             WHERE p.workspace_id = ?1 AND m.id = ?2",
            rusqlite::params![workspace_id, milestone_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| CoreError::not_found("Milestone"))?;
    let name = validate_name("name", &input.name)?;
    validate_date_range(None, input.due_date.as_deref())?;
    conn.execute(
        "UPDATE milestone SET name = ?1, description = ?2, due_date = ?3, status = ?4, \
         updated_at = ?5 WHERE project_id = ?6 AND id = ?7",
        rusqlite::params![
            name,
            input.description,
            input.due_date,
            input.status.unwrap_or(MilestoneStatus::Planned).as_str(),
            now_rfc3339(),
            project_id,
            milestone_id,
        ],
    )?;
    get_milestone(conn, milestone_id)
}

pub fn assign_milestone_issue(
    conn: &Connection,
    workspace_id: &str,
    milestone_id: &str,
    issue_id: &str,
    assigned: bool,
) -> CoreResult<()> {
    let project_id: String = conn
        .query_row(
            "SELECT m.project_id FROM milestone m JOIN project p ON p.id = m.project_id \
             WHERE p.workspace_id = ?1 AND m.id = ?2",
            rusqlite::params![workspace_id, milestone_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| CoreError::not_found("Milestone"))?;
    let issue_project: Option<String> = conn
        .query_row(
            "SELECT project_id FROM issue WHERE workspace_id = ?1 AND id = ?2 AND deleted_at IS NULL",
            rusqlite::params![workspace_id, issue_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| CoreError::not_found("Issue"))?;
    if issue_project.as_deref() != Some(project_id.as_str()) {
        return Err(CoreError::validation(
            "issueId",
            "Only issues in the same project can be linked to this milestone.",
        ));
    }
    if assigned {
        conn.execute(
            "INSERT INTO issue_milestone (issue_id, milestone_id, added_at) VALUES (?1, ?2, ?3) \
             ON CONFLICT(issue_id) DO UPDATE SET milestone_id = excluded.milestone_id, added_at = excluded.added_at",
            rusqlite::params![issue_id, milestone_id, now_rfc3339()],
        )?;
    } else {
        conn.execute(
            "DELETE FROM issue_milestone WHERE issue_id = ?1 AND milestone_id = ?2",
            rusqlite::params![issue_id, milestone_id],
        )?;
    }
    Ok(())
}

pub fn list_initiatives(conn: &Connection, workspace_id: &str) -> CoreResult<Vec<Initiative>> {
    let mut statement = conn.prepare(
        "SELECT i.id, i.name, i.description, i.status, i.priority, i.start_date, i.target_date, \
         i.created_at, i.updated_at, i.archived_at, \
         (SELECT count(*) FROM project p WHERE p.initiative_id = i.id AND p.archived_at IS NULL), \
         (SELECT count(*) FROM issue x JOIN workflow_status s ON s.id = x.status_id \
            JOIN project p ON p.id = x.project_id WHERE p.initiative_id = i.id \
            AND x.deleted_at IS NULL AND x.archived_at IS NULL), \
         (SELECT count(*) FROM issue x JOIN workflow_status s ON s.id = x.status_id \
            JOIN project p ON p.id = x.project_id WHERE p.initiative_id = i.id \
            AND x.deleted_at IS NULL AND x.archived_at IS NULL AND s.category = 'completed'), \
         (SELECT count(*) FROM issue x JOIN workflow_status s ON s.id = x.status_id \
            JOIN project p ON p.id = x.project_id WHERE p.initiative_id = i.id \
            AND x.deleted_at IS NULL AND x.archived_at IS NULL AND s.category = 'canceled') \
         FROM initiative i WHERE i.workspace_id = ?1 AND i.archived_at IS NULL \
         ORDER BY i.updated_at DESC",
    )?;
    let rows = statement.query_map([workspace_id], read_initiative)?;
    rows.map(|row| row.map_err(CoreError::from))
        .collect::<CoreResult<Vec<_>>>()
}

pub fn initiative_detail(
    conn: &Connection,
    workspace_id: &str,
    initiative_id: &str,
) -> CoreResult<InitiativeDetail> {
    let initiative = get_initiative(conn, workspace_id, initiative_id)?;
    let projects = list_projects(
        conn,
        workspace_id,
        &ProjectQuery {
            initiative_id: Some(initiative_id.to_owned()),
            ..Default::default()
        },
    )?;
    Ok(InitiativeDetail {
        initiative,
        projects,
    })
}

pub fn create_initiative(
    conn: &Connection,
    workspace_id: &str,
    input: &InitiativeInput,
) -> CoreResult<Initiative> {
    let name = validate_name("name", &input.name)?;
    validate_date_range(input.start_date.as_deref(), input.target_date.as_deref())?;
    let id = uuid::Uuid::now_v7().to_string();
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO initiative \
         (id, workspace_id, name, description, status, priority, start_date, target_date, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        rusqlite::params![
            id,
            workspace_id,
            name,
            input.description,
            input.status.unwrap_or(PlanningStatus::Planned).as_str(),
            input.priority.unwrap_or(Priority::None).as_str(),
            input.start_date,
            input.target_date,
            now,
        ],
    )?;
    get_initiative(conn, workspace_id, &id)
}

pub fn update_initiative(
    conn: &Connection,
    workspace_id: &str,
    initiative_id: &str,
    input: &InitiativeInput,
) -> CoreResult<Initiative> {
    let name = validate_name("name", &input.name)?;
    validate_date_range(input.start_date.as_deref(), input.target_date.as_deref())?;
    let changed = conn.execute(
        "UPDATE initiative SET name = ?1, description = ?2, status = ?3, priority = ?4, \
         start_date = ?5, target_date = ?6, updated_at = ?7 WHERE workspace_id = ?8 AND id = ?9",
        rusqlite::params![
            name,
            input.description,
            input.status.unwrap_or(PlanningStatus::Planned).as_str(),
            input.priority.unwrap_or(Priority::None).as_str(),
            input.start_date,
            input.target_date,
            now_rfc3339(),
            workspace_id,
            initiative_id,
        ],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Initiative"));
    }
    get_initiative(conn, workspace_id, initiative_id)
}

pub fn list_cycles(conn: &Connection, workspace_id: &str) -> CoreResult<Vec<Cycle>> {
    let mut statement = conn.prepare(
        "SELECT id, name, start_date, end_date, created_at, updated_at FROM cycle \
         WHERE workspace_id = ?1 AND archived_at IS NULL ORDER BY start_date DESC",
    )?;
    let rows = statement.query_map([workspace_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
        ))
    })?;
    let mut cycles = Vec::new();
    for row in rows {
        let (id, name, start_date, end_date, created_at, updated_at) = row?;
        cycles.push(build_cycle(
            conn, id, name, start_date, end_date, created_at, updated_at,
        )?);
    }
    Ok(cycles)
}

pub fn create_cycle(
    conn: &Connection,
    workspace_id: &str,
    input: &CycleInput,
) -> CoreResult<Cycle> {
    let name = validate_name("name", &input.name)?;
    validate_cycle_dates(&input.start_date, &input.end_date)?;
    let id = uuid::Uuid::now_v7().to_string();
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO cycle (id, workspace_id, name, start_date, end_date, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        rusqlite::params![
            id,
            workspace_id,
            name,
            input.start_date,
            input.end_date,
            now
        ],
    )?;
    get_cycle(conn, workspace_id, &id)
}

pub fn update_cycle(
    conn: &Connection,
    workspace_id: &str,
    cycle_id: &str,
    input: &CycleInput,
) -> CoreResult<Cycle> {
    let name = validate_name("name", &input.name)?;
    validate_cycle_dates(&input.start_date, &input.end_date)?;
    let changed = conn.execute(
        "UPDATE cycle SET name = ?1, start_date = ?2, end_date = ?3, updated_at = ?4 \
         WHERE workspace_id = ?5 AND id = ?6",
        rusqlite::params![
            name,
            input.start_date,
            input.end_date,
            now_rfc3339(),
            workspace_id,
            cycle_id,
        ],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Cycle"));
    }
    get_cycle(conn, workspace_id, cycle_id)
}

pub fn cycle_detail(
    conn: &Connection,
    workspace_id: &str,
    cycle_id: &str,
) -> CoreResult<CycleDetail> {
    let cycle = get_cycle(conn, workspace_id, cycle_id)?;
    let issues = issues::list_by_cycle(conn, workspace_id, cycle_id)?;
    Ok(CycleDetail { cycle, issues })
}

pub fn assign_cycle_issue(
    conn: &Connection,
    workspace_id: &str,
    cycle_id: &str,
    issue_id: &str,
    assigned: bool,
) -> CoreResult<()> {
    get_cycle(conn, workspace_id, cycle_id)?;
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM issue WHERE workspace_id = ?1 AND id = ?2 AND deleted_at IS NULL",
            rusqlite::params![workspace_id, issue_id],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);
    if !exists {
        return Err(CoreError::not_found("Issue"));
    }
    if assigned {
        conn.execute(
            "INSERT INTO issue_cycle (issue_id, cycle_id, added_at) VALUES (?1, ?2, ?3) \
             ON CONFLICT(issue_id) DO UPDATE SET cycle_id = excluded.cycle_id, added_at = excluded.added_at",
            rusqlite::params![issue_id, cycle_id, now_rfc3339()],
        )?;
    } else {
        conn.execute(
            "DELETE FROM issue_cycle WHERE issue_id = ?1 AND cycle_id = ?2",
            rusqlite::params![issue_id, cycle_id],
        )?;
    }
    Ok(())
}

pub fn list_documents(conn: &Connection, workspace_id: &str) -> CoreResult<Vec<Document>> {
    let mut statement = conn.prepare(
        "SELECT d.id, d.title, d.body, d.project_id, d.initiative_id, p.name, i.name, \
         d.archived_at, d.created_at, d.updated_at \
         FROM document d LEFT JOIN project p ON p.id = d.project_id \
         LEFT JOIN initiative i ON i.id = d.initiative_id \
         WHERE d.workspace_id = ?1 AND d.archived_at IS NULL ORDER BY d.updated_at DESC",
    )?;
    let rows = statement.query_map([workspace_id], read_document)?;
    rows.map(|row| row.map_err(CoreError::from))
        .collect::<CoreResult<Vec<_>>>()
}

pub fn get_document(
    conn: &Connection,
    workspace_id: &str,
    document_id: &str,
) -> CoreResult<Document> {
    conn.query_row(
        "SELECT d.id, d.title, d.body, d.project_id, d.initiative_id, p.name, i.name, \
         d.archived_at, d.created_at, d.updated_at \
         FROM document d LEFT JOIN project p ON p.id = d.project_id \
         LEFT JOIN initiative i ON i.id = d.initiative_id \
         WHERE d.workspace_id = ?1 AND d.id = ?2",
        rusqlite::params![workspace_id, document_id],
        read_document,
    )
    .optional()?
    .ok_or_else(|| CoreError::not_found("Document"))
}

pub fn create_document(
    conn: &Connection,
    workspace_id: &str,
    input: &DocumentInput,
) -> CoreResult<Document> {
    let title = validate_document_title(&input.title)?;
    let body = validate_body(&input.body)?;
    validate_document_links(conn, workspace_id, input)?;
    let id = uuid::Uuid::now_v7().to_string();
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO document \
         (id, workspace_id, title, body, project_id, initiative_id, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        rusqlite::params![
            id,
            workspace_id,
            title,
            body,
            input.project_id,
            input.initiative_id,
            now,
        ],
    )?;
    get_document(conn, workspace_id, &id)
}

pub fn update_document(
    conn: &Connection,
    workspace_id: &str,
    document_id: &str,
    input: &DocumentInput,
) -> CoreResult<Document> {
    let title = validate_document_title(&input.title)?;
    let body = validate_body(&input.body)?;
    validate_document_links(conn, workspace_id, input)?;
    let changed = conn.execute(
        "UPDATE document SET title = ?1, body = ?2, project_id = ?3, initiative_id = ?4, \
         updated_at = ?5 WHERE workspace_id = ?6 AND id = ?7",
        rusqlite::params![
            title,
            body,
            input.project_id,
            input.initiative_id,
            now_rfc3339(),
            workspace_id,
            document_id,
        ],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Document"));
    }
    get_document(conn, workspace_id, document_id)
}

pub fn delete_document(conn: &Connection, workspace_id: &str, document_id: &str) -> CoreResult<()> {
    let changed = conn.execute(
        "DELETE FROM document WHERE workspace_id = ?1 AND id = ?2",
        rusqlite::params![workspace_id, document_id],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Document"));
    }
    Ok(())
}

pub fn export_document(
    conn: &Connection,
    workspace_id: &str,
    document_id: &str,
) -> CoreResult<String> {
    let document = get_document(conn, workspace_id, document_id)?;
    Ok(markdown_export(&document))
}

fn get_milestone(conn: &Connection, milestone_id: &str) -> CoreResult<Milestone> {
    conn.query_row(
        &format!("{MILESTONE_SELECT} WHERE m.id = ?1"),
        [milestone_id],
        read_milestone,
    )
    .optional()?
    .ok_or_else(|| CoreError::not_found("Milestone"))
}

fn get_initiative(
    conn: &Connection,
    workspace_id: &str,
    initiative_id: &str,
) -> CoreResult<Initiative> {
    list_initiatives(conn, workspace_id)?
        .into_iter()
        .find(|initiative| initiative.id == initiative_id)
        .ok_or_else(|| CoreError::not_found("Initiative"))
}

fn get_cycle(conn: &Connection, workspace_id: &str, cycle_id: &str) -> CoreResult<Cycle> {
    let row = conn
        .query_row(
            "SELECT id, name, start_date, end_date, created_at, updated_at FROM cycle \
             WHERE workspace_id = ?1 AND id = ?2 AND archived_at IS NULL",
            rusqlite::params![workspace_id, cycle_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| CoreError::not_found("Cycle"))?;
    build_cycle(conn, row.0, row.1, row.2, row.3, row.4, row.5)
}

fn build_cycle(
    conn: &Connection,
    id: String,
    name: String,
    start_date: String,
    end_date: String,
    created_at: String,
    updated_at: String,
) -> CoreResult<Cycle> {
    let today = Utc::now().date_naive();
    let start = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|_| CoreError::internal("Cycle start date is invalid."))?;
    let end = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|_| CoreError::internal("Cycle end date is invalid."))?;
    let status = if today > end {
        CycleStatus::Completed
    } else if today < start {
        CycleStatus::Upcoming
    } else {
        CycleStatus::Active
    };
    let days_remaining = (end - today).num_days().max(0);
    let (issue_count, completed_count, total_estimate, completed_estimate, carry_over_count) =
        conn.query_row(
            "SELECT count(x.id), \
                    coalesce(sum(CASE WHEN s.category = 'completed' THEN 1 ELSE 0 END), 0), \
                    coalesce(sum(x.estimate), 0), \
                    coalesce(sum(CASE WHEN s.category = 'completed' THEN x.estimate ELSE 0 END), 0), \
                    coalesce(sum(CASE WHEN s.category NOT IN ('completed', 'canceled') \
                        AND x.due_date IS NOT NULL AND x.due_date < ?2 THEN 1 ELSE 0 END), 0) \
             FROM issue_cycle ic JOIN issue x ON x.id = ic.issue_id \
             JOIN workflow_status s ON s.id = x.status_id \
             WHERE ic.cycle_id = ?1 AND x.deleted_at IS NULL AND x.archived_at IS NULL",
            rusqlite::params![id, today.format("%Y-%m-%d").to_string()],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, f64>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        )?;
    Ok(Cycle {
        id,
        name,
        start_date,
        end_date,
        status,
        summary: CycleSummary {
            issue_count,
            completed_count,
            total_estimate,
            completed_estimate,
            carry_over_count,
            days_remaining,
        },
        created_at,
        updated_at,
    })
}

fn read_project(row: &Row<'_>) -> rusqlite::Result<Project> {
    let total: i64 = row.get(13)?;
    let completed: i64 = row.get(14)?;
    let canceled: i64 = row.get(15)?;
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        status: PlanningStatus::parse(&row.get::<_, String>(3)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        priority: Priority::parse(&row.get::<_, String>(4)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        start_date: row.get(5)?,
        target_date: row.get(6)?,
        color: row.get(7)?,
        archived_at: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        initiative: match (
            row.get::<_, Option<String>>(11)?,
            row.get::<_, Option<String>>(12)?,
        ) {
            (Some(id), Some(name)) => Some(InitiativeRef { id, name }),
            _ => None,
        },
        progress: Progress::new(completed, total, canceled),
        issue_key_prefix: row.get(16)?,
    })
}

fn read_milestone(row: &Row<'_>) -> rusqlite::Result<Milestone> {
    let total: i64 = row.get(8)?;
    let completed: i64 = row.get(9)?;
    let canceled: i64 = row.get(10)?;
    Ok(Milestone {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        due_date: row.get(4)?,
        status: MilestoneStatus::parse(&row.get::<_, String>(5)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        progress: Progress::new(completed, total, canceled),
    })
}

fn read_initiative(row: &Row<'_>) -> rusqlite::Result<Initiative> {
    let project_count: i64 = row.get(10)?;
    let total: i64 = row.get(11)?;
    let completed: i64 = row.get(12)?;
    let canceled: i64 = row.get(13)?;
    Ok(Initiative {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        status: PlanningStatus::parse(&row.get::<_, String>(3)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        priority: Priority::parse(&row.get::<_, String>(4)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        start_date: row.get(5)?,
        target_date: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        archived_at: row.get(9)?,
        progress: Progress::new(completed, total, canceled),
        project_count,
    })
}

fn read_document(row: &Row<'_>) -> rusqlite::Result<Document> {
    Ok(Document {
        id: row.get(0)?,
        title: row.get(1)?,
        body: row.get(2)?,
        project_id: row.get(3)?,
        initiative_id: row.get(4)?,
        project_name: row.get(5)?,
        initiative_name: row.get(6)?,
        archived_at: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn read_status(row: &Row<'_>, offset: usize) -> rusqlite::Result<StatusSummary> {
    Ok(StatusSummary {
        id: row.get(offset)?,
        name: row.get(offset + 1)?,
        category: crate::domain::issue::StatusCategory::parse(&row.get::<_, String>(offset + 2)?)
            .map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        color: crate::domain::issue::StatusColor::parse(&row.get::<_, String>(offset + 3)?)
            .map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?,
        position: row.get(offset + 4)?,
        system_key: row.get(offset + 5)?,
    })
}

fn normalize_project_key_prefix(raw: Option<&str>) -> CoreResult<Option<String>> {
    match raw.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => Ok(Some(Workspace::validate_key_prefix(value)?)),
        None => Ok(None),
    }
}

fn ensure_initiative(conn: &Connection, workspace_id: &str, initiative_id: &str) -> CoreResult<()> {
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM initiative WHERE workspace_id = ?1 AND id = ?2 AND archived_at IS NULL",
            rusqlite::params![workspace_id, initiative_id],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);
    if exists {
        Ok(())
    } else {
        Err(CoreError::not_found("Initiative"))
    }
}

fn validate_document_links(
    conn: &Connection,
    workspace_id: &str,
    input: &DocumentInput,
) -> CoreResult<()> {
    if let Some(project_id) = &input.project_id {
        get_project(conn, workspace_id, project_id)?;
    }
    if let Some(initiative_id) = &input.initiative_id {
        ensure_initiative(conn, workspace_id, initiative_id)?;
    }
    Ok(())
}

impl PlanningStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Canceled => "canceled",
        }
    }
}

impl MilestoneStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Canceled => "canceled",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repositories::{issues, workflow};
    use crate::db::Database;
    use crate::domain::issue::{CreateIssueInput, IssueType, Priority, UpdateIssueInput};
    use crate::domain::planning::{InitiativeInput, ProjectQuery};

    fn fixture() -> (tempfile::TempDir, Database, String) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("open");
        let workspace = db
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "Planning"))
            .expect("workspace");
        (dir, db, workspace.id)
    }

    fn issue(conn: &Connection, workspace_id: &str, project_id: &str, title: &str) -> String {
        issues::create(
            conn,
            workspace_id,
            &CreateIssueInput {
                title: title.into(),
                description: String::new(),
                status_id: None,
                priority: Priority::Medium,
                issue_type: IssueType::Task,
                estimate: Some(2.0),
                start_date: None,
                due_date: None,
                project_id: Some(project_id.into()),
                parent_id: None,
                label_names: vec![],
            },
        )
        .expect("issue")
    }

    fn complete(conn: &Connection, workspace_id: &str, issue_id: &str) {
        let completed = workflow::statuses(conn, workspace_id)
            .expect("statuses")
            .into_iter()
            .find(|status| status.category == crate::domain::issue::StatusCategory::Completed)
            .expect("completed status");
        issues::update(
            conn,
            workspace_id,
            issue_id,
            &UpdateIssueInput {
                status_id: Some(completed.id),
                ..Default::default()
            },
        )
        .expect("complete issue");
    }

    #[test]
    fn project_milestone_initiative_and_cycle_progress() {
        let (_dir, db, workspace_id) = fixture();
        let (project, initiative) = db
            .with_conn(|conn| {
                let initiative = create_initiative(
                    conn,
                    &workspace_id,
                    &InitiativeInput {
                        name: "Launch".into(),
                        ..Default::default()
                    },
                )?;
                let project = create_project(
                    conn,
                    &workspace_id,
                    &ProjectInput {
                        name: "Desktop".into(),
                        initiative_id: Some(initiative.id.clone()),
                        ..Default::default()
                    },
                )?;
                let milestone = create_milestone(
                    conn,
                    &workspace_id,
                    &project.id,
                    &MilestoneInput {
                        name: "Beta".into(),
                        ..Default::default()
                    },
                )?;
                let one = issue(conn, &workspace_id, &project.id, "One");
                let two = issue(conn, &workspace_id, &project.id, "Two");
                let canceled = issue(conn, &workspace_id, &project.id, "Canceled");
                complete(conn, &workspace_id, &one);
                let canceled_status = workflow::statuses(conn, &workspace_id)?
                    .into_iter()
                    .find(|status| {
                        status.category == crate::domain::issue::StatusCategory::Canceled
                    })
                    .expect("canceled status");
                issues::update(
                    conn,
                    &workspace_id,
                    &canceled,
                    &UpdateIssueInput {
                        status_id: Some(canceled_status.id),
                        ..Default::default()
                    },
                )?;
                assign_milestone_issue(conn, &workspace_id, &milestone.id, &one, true)?;
                assign_milestone_issue(conn, &workspace_id, &milestone.id, &two, true)?;
                Ok((project, initiative))
            })
            .expect("seed");

        let detail = db
            .with_conn(|conn| project_detail(conn, &workspace_id, &project.id))
            .expect("project detail");
        assert_eq!(detail.project.progress.total, 2);
        assert_eq!(detail.project.progress.completed, 1);
        assert_eq!(detail.project.progress.percent, 50.0);
        assert_eq!(detail.milestones.len(), 1);
        assert_eq!(detail.milestones[0].progress.percent, 50.0);
        assert!(detail.issue_counts.iter().any(|count| count.count == 1));

        let initiative_detail = db
            .with_conn(|conn| initiative_detail(conn, &workspace_id, &initiative.id))
            .expect("initiative detail");
        assert_eq!(initiative_detail.projects.len(), 1);
        assert_eq!(initiative_detail.initiative.progress.percent, 50.0);

        let today = Utc::now().date_naive();
        let cycle = db
            .with_conn(|conn| {
                let cycle = create_cycle(
                    conn,
                    &workspace_id,
                    &CycleInput {
                        name: "Cycle 1".into(),
                        start_date: (today - chrono::Duration::days(1)).to_string(),
                        end_date: (today + chrono::Duration::days(5)).to_string(),
                    },
                )?;
                let issue_id = issue(conn, &workspace_id, &project.id, "Cycle task");
                assign_cycle_issue(conn, &workspace_id, &cycle.id, &issue_id, true)?;
                Ok(cycle)
            })
            .expect("cycle");
        let cycle_detail = db
            .with_conn(|conn| cycle_detail(conn, &workspace_id, &cycle.id))
            .expect("cycle detail");
        assert_eq!(cycle_detail.issues.len(), 1);
        assert_eq!(cycle_detail.cycle.summary.issue_count, 1);
        assert!(cycle_detail.cycle.summary.days_remaining > 0);

        let document = db
            .with_conn(|conn| {
                create_document(
                    conn,
                    &workspace_id,
                    &DocumentInput {
                        title: "Release notes".into(),
                        body: "Local-first.".into(),
                        project_id: Some(project.id.clone()),
                        initiative_id: None,
                    },
                )
            })
            .expect("document");
        let markdown = db
            .with_conn(|conn| export_document(conn, &workspace_id, &document.id))
            .expect("export");
        assert!(markdown.starts_with("# Release notes"));
        assert!(markdown.contains("Local-first."));
    }

    #[test]
    fn project_query_filters_by_initiative_and_search() {
        let (_dir, db, workspace_id) = fixture();
        let (project, initiative) = db
            .with_conn(|conn| {
                let initiative = create_initiative(
                    conn,
                    &workspace_id,
                    &InitiativeInput {
                        name: "Initiative".into(),
                        ..Default::default()
                    },
                )?;
                let project = create_project(
                    conn,
                    &workspace_id,
                    &ProjectInput {
                        name: "Visible project".into(),
                        initiative_id: Some(initiative.id.clone()),
                        ..Default::default()
                    },
                )?;
                create_project(
                    conn,
                    &workspace_id,
                    &ProjectInput {
                        name: "Hidden project".into(),
                        description: "Searchable project description".into(),
                        ..Default::default()
                    },
                )?;
                Ok((project, initiative))
            })
            .expect("projects");

        let results = db
            .with_conn(|conn| {
                list_projects(
                    conn,
                    &workspace_id,
                    &ProjectQuery {
                        search: Some("visible".into()),
                        initiative_id: Some(initiative.id.clone()),
                        ..Default::default()
                    },
                )
            })
            .expect("query");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, project.id);

        let description_results = db
            .with_conn(|conn| {
                list_projects(
                    conn,
                    &workspace_id,
                    &ProjectQuery {
                        search: Some("searchable".into()),
                        ..Default::default()
                    },
                )
            })
            .expect("description search");
        assert_eq!(description_results.len(), 1);
        assert_eq!(description_results[0].name, "Hidden project");
    }

    #[test]
    fn project_prefix_applies_to_new_issues_without_rewriting_old_keys() {
        let (_dir, db, workspace_id) = fixture();
        db.with_conn(|conn| {
            let project = create_project(
                conn,
                &workspace_id,
                &ProjectInput {
                    name: "Prefixed".into(),
                    issue_key_prefix: Some("prj".into()),
                    ..Default::default()
                },
            )?;
            let first_id = issue(conn, &workspace_id, &project.id, "First");
            let first = issues::get_summary(conn, &workspace_id, &first_id)?;
            assert_eq!(first.key, "PRJ-1");

            update_project(
                conn,
                &workspace_id,
                &project.id,
                &ProjectInput {
                    name: "Prefixed".into(),
                    issue_key_prefix: Some("px".into()),
                    ..Default::default()
                },
            )?;
            let second_id = issue(conn, &workspace_id, &project.id, "Second");
            let second = issues::get_summary(conn, &workspace_id, &second_id)?;
            assert_eq!(second.key, "PX-2");
            let first_after = issues::get_summary(conn, &workspace_id, &first_id)?;
            assert_eq!(first_after.key, "PRJ-1");
            Ok(())
        })
        .expect("project prefix behavior");
    }
}
