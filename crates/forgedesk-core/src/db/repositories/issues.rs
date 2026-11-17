use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use rusqlite::types::Value as SqlValue;
use rusqlite::{params_from_iter, Connection, OptionalExtension, Row};
use serde_json::json;

use crate::domain::time::now_rfc3339;
use crate::domain::{
    validate_date, validate_description, validate_estimate, validate_title, ActivityEvent,
    BatchIssueInput, CreateIssueInput, IssueDetail, IssueQuery, IssueRelation, IssueSource,
    IssueSummary, IssueType, Label, Priority, ProjectRef, RelationInputKind, StatusCategory,
    UpdateIssueInput,
};
use crate::error::{CoreError, CoreResult};

use super::{activity, comments, issue_query, labels, projects, relations, workflow};

const SUMMARY_SELECT: &str = "\
SELECT i.id, i.key, i.key_number, i.title, \
       s.id, s.name, s.category, s.color, s.position, s.system_key, \
       i.priority, i.issue_type, i.estimate, i.start_date, i.due_date, \
       i.parent_id, i.source, i.remote_url, i.archived_at, i.created_at, i.updated_at, \
       i.completed_at, p.id, p.name, \
       (SELECT count(*) FROM issue c WHERE c.parent_id = i.id AND c.deleted_at IS NULL) \
FROM issue i \
JOIN workflow_status s ON s.id = i.status_id \
LEFT JOIN project p ON p.id = i.project_id";

struct RawSummary {
    id: String,
    key: String,
    key_number: i64,
    title: String,
    status_id: String,
    status_name: String,
    status_category: String,
    status_color: String,
    status_position: i64,
    status_system_key: Option<String>,
    priority: String,
    issue_type: String,
    estimate: Option<f64>,
    start_date: Option<String>,
    due_date: Option<String>,
    parent_id: Option<String>,
    source: String,
    remote_url: Option<String>,
    archived_at: Option<String>,
    created_at: String,
    updated_at: String,
    completed_at: Option<String>,
    project_id: Option<String>,
    project_name: Option<String>,
    sub_issue_count: i64,
}

fn read_raw(row: &Row<'_>) -> rusqlite::Result<RawSummary> {
    Ok(RawSummary {
        id: row.get(0)?,
        key: row.get(1)?,
        key_number: row.get(2)?,
        title: row.get(3)?,
        status_id: row.get(4)?,
        status_name: row.get(5)?,
        status_category: row.get(6)?,
        status_color: row.get(7)?,
        status_position: row.get(8)?,
        status_system_key: row.get(9)?,
        priority: row.get(10)?,
        issue_type: row.get(11)?,
        estimate: row.get(12)?,
        start_date: row.get(13)?,
        due_date: row.get(14)?,
        parent_id: row.get(15)?,
        source: row.get(16)?,
        remote_url: row.get(17)?,
        archived_at: row.get(18)?,
        created_at: row.get(19)?,
        updated_at: row.get(20)?,
        completed_at: row.get(21)?,
        project_id: row.get(22)?,
        project_name: row.get(23)?,
        sub_issue_count: row.get(24)?,
    })
}

fn parse_summary(raw: RawSummary) -> CoreResult<IssueSummary> {
    let project = match (raw.project_id, raw.project_name) {
        (Some(id), Some(name)) => Some(ProjectRef { id, name }),
        _ => None,
    };
    Ok(IssueSummary {
        id: raw.id,
        key: raw.key,
        key_number: raw.key_number,
        title: raw.title,
        status: workflow::parse_status(
            raw.status_id,
            raw.status_name,
            &raw.status_category,
            &raw.status_color,
            raw.status_position,
            raw.status_system_key,
        )?,
        priority: Priority::parse(&raw.priority)?,
        issue_type: IssueType::parse(&raw.issue_type)?,
        project,
        labels: Vec::new(),
        estimate: raw.estimate,
        start_date: raw.start_date,
        due_date: raw.due_date,
        parent_id: raw.parent_id,
        source: IssueSource::parse(&raw.source)?,
        remote_url: raw.remote_url,
        archived_at: raw.archived_at,
        created_at: raw.created_at,
        updated_at: raw.updated_at,
        completed_at: raw.completed_at,
        sub_issue_count: raw.sub_issue_count,
    })
}

fn query_summaries(
    conn: &Connection,
    sql: &str,
    params: &[SqlValue],
) -> CoreResult<Vec<IssueSummary>> {
    let mut statement = conn.prepare(sql)?;
    let raw_rows = statement
        .query_map(params_from_iter(params.iter()), read_raw)?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);

    let mut summaries = raw_rows
        .into_iter()
        .map(parse_summary)
        .collect::<CoreResult<Vec<_>>>()?;
    let ids = summaries
        .iter()
        .map(|summary| summary.id.clone())
        .collect::<Vec<_>>();
    let mut label_map = labels::labels_for_issues(conn, &ids)?;
    for summary in &mut summaries {
        summary.labels = label_map.remove(&summary.id).unwrap_or_default();
    }
    Ok(summaries)
}

pub fn list_by_cycle(
    conn: &Connection,
    workspace_id: &str,
    cycle_id: &str,
) -> CoreResult<Vec<IssueSummary>> {
    let sql = format!(
        "{SUMMARY_SELECT} JOIN issue_cycle ic ON ic.issue_id = i.id \
         WHERE i.workspace_id = ? AND ic.cycle_id = ? \
           AND i.deleted_at IS NULL AND i.archived_at IS NULL \
         ORDER BY s.position ASC, i.updated_at DESC"
    );
    query_summaries(
        conn,
        &sql,
        &[
            SqlValue::Text(workspace_id.to_owned()),
            SqlValue::Text(cycle_id.to_owned()),
        ],
    )
}

pub fn get_summary(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
) -> CoreResult<IssueSummary> {
    let sql = format!("{SUMMARY_SELECT} WHERE i.workspace_id = ? AND i.id = ?");
    query_summaries(
        conn,
        &sql,
        &[
            SqlValue::Text(workspace_id.to_owned()),
            SqlValue::Text(issue_id.to_owned()),
        ],
    )?
    .into_iter()
    .next()
    .ok_or_else(|| CoreError::not_found("Issue"))
}

pub fn get_detail(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
) -> CoreResult<IssueDetail> {
    let summary = get_summary(conn, workspace_id, issue_id)?;
    let description = conn
        .query_row(
            "SELECT description FROM issue WHERE id = ?1 AND workspace_id = ?2",
            rusqlite::params![issue_id, workspace_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or_else(|| CoreError::not_found("Issue"))?;

    let parent = match &summary.parent_id {
        Some(parent_id) => get_summary(conn, workspace_id, parent_id).ok(),
        None => None,
    };

    let children_sql = format!(
        "{SUMMARY_SELECT} WHERE i.parent_id = ? AND i.deleted_at IS NULL \
         ORDER BY i.key_number ASC"
    );
    let children = query_summaries(conn, &children_sql, &[SqlValue::Text(issue_id.to_owned())])?;

    Ok(IssueDetail {
        summary,
        description,
        parent,
        children,
        relations: relations::list_for_issue(conn, issue_id)?,
        comments: comments::list(conn, issue_id)?,
        activity: activity::list(conn, issue_id)?,
    })
}

pub fn list(
    conn: &Connection,
    workspace_id: &str,
    query: &IssueQuery,
) -> CoreResult<Vec<IssueSummary>> {
    let built = issue_query::build(workspace_id, query)?;
    let sql = format!(
        "{SUMMARY_SELECT} WHERE {} {} LIMIT ? OFFSET ?",
        built.where_clause, built.order_clause
    );
    let mut params = built.params;
    params.push(SqlValue::Integer(built.limit));
    params.push(SqlValue::Integer(built.offset));
    query_summaries(conn, &sql, &params)
}

pub fn create(
    conn: &Connection,
    workspace_id: &str,
    input: &CreateIssueInput,
) -> CoreResult<String> {
    input.validate()?;

    let status = match &input.status_id {
        Some(status_id) => workflow::find_status(conn, workspace_id, status_id)?
            .ok_or_else(|| CoreError::validation("statusId", "Unknown workflow status."))?,
        None => workflow::default_status(conn, workspace_id)?,
    };
    let project = match &input.project_id {
        Some(project_id) => Some(
            projects::find(conn, workspace_id, project_id)?
                .ok_or_else(|| CoreError::validation("projectId", "Unknown project."))?,
        ),
        None => None,
    };
    let label_rows = labels::ensure_many(conn, workspace_id, &input.label_names)?;

    let id = uuid::Uuid::now_v7().to_string();
    if let Some(parent_id) = &input.parent_id {
        ensure_parent_valid(conn, workspace_id, &id, Some(parent_id))?;
    }

    let (key_number, key) = crate::db::repositories::workspace::next_issue_key(
        conn,
        workspace_id,
        project.as_ref().map(|value| value.id.as_str()),
    )?;
    let now = now_rfc3339();
    let completed_at = if status.category == StatusCategory::Completed {
        Some(now.clone())
    } else {
        None
    };

    conn.execute(
        "INSERT INTO issue \
         (id, workspace_id, key_number, key, title, description, status_id, priority, issue_type, \
          estimate, start_date, due_date, project_id, parent_id, source, remote_url, archived_at, \
          deleted_at, created_at, updated_at, completed_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, \
                 'local', NULL, NULL, NULL, ?15, ?15, ?16)",
        rusqlite::params![
            id,
            workspace_id,
            key_number,
            key,
            input.normalized_title(),
            input.description,
            status.id,
            input.priority.as_str(),
            input.issue_type.as_str(),
            input.estimate,
            input.start_date,
            input.due_date,
            project.as_ref().map(|value| value.id.as_str()),
            input.parent_id,
            now,
            completed_at,
        ],
    )?;

    labels::set_issue_labels(conn, &id, &label_rows)?;
    activity::record(
        conn,
        &id,
        "issue_created",
        &json!({"key": key, "title": input.normalized_title()}),
    )?;
    Ok(id)
}

pub fn update(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
    input: &UpdateIssueInput,
) -> CoreResult<IssueDetail> {
    update_inner(conn, workspace_id, issue_id, input, true)
}

/// Apply provider-originated changes without echoing them into the outbound queue.
pub fn update_from_remote(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
    input: &UpdateIssueInput,
) -> CoreResult<IssueDetail> {
    update_inner(conn, workspace_id, issue_id, input, false)
}

fn update_inner(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
    input: &UpdateIssueInput,
    enqueue_remote_changes: bool,
) -> CoreResult<IssueDetail> {
    input.validate()?;
    let current = get_summary(conn, workspace_id, issue_id)?;
    let current_description = conn
        .query_row(
            "SELECT description FROM issue WHERE id = ?1 AND workspace_id = ?2",
            rusqlite::params![issue_id, workspace_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or_else(|| CoreError::not_found("Issue"))?;
    let current_labels = labels::labels_for_issues(conn, &[issue_id.to_owned()])?
        .remove(issue_id)
        .unwrap_or_default();

    let status = match &input.status_id {
        Some(status_id) => workflow::find_status(conn, workspace_id, status_id)?
            .ok_or_else(|| CoreError::validation("statusId", "Unknown workflow status."))?,
        None => current.status.clone(),
    };

    let title = match &input.title {
        Some(title) => validate_title(title)?,
        None => current.title.clone(),
    };
    let description = input
        .description
        .clone()
        .unwrap_or_else(|| current_description.clone());
    validate_description(&description)?;

    let estimate = if clear_field(input, "estimate") {
        None
    } else {
        input.estimate.or(current.estimate)
    };
    if let Some(estimate) = estimate {
        validate_estimate(estimate)?;
    }
    let start_date = if clear_field(input, "startDate") {
        None
    } else {
        input.start_date.clone().or(current.start_date.clone())
    };
    let due_date = if clear_field(input, "dueDate") {
        None
    } else {
        input.due_date.clone().or(current.due_date.clone())
    };
    if let Some(date) = &start_date {
        validate_date("startDate", date)?;
    }
    if let Some(date) = &due_date {
        validate_date("dueDate", date)?;
    }
    if let (Some(start), Some(due)) = (&start_date, &due_date) {
        if start > due {
            return Err(CoreError::validation(
                "dueDate",
                "Due date cannot be earlier than the start date.",
            ));
        }
    }

    let project = if clear_field(input, "projectId") {
        None
    } else if let Some(project_id) = &input.project_id {
        Some(
            projects::find(conn, workspace_id, project_id)?
                .ok_or_else(|| CoreError::validation("projectId", "Unknown project."))?,
        )
    } else {
        current.project.clone()
    };

    let parent_id = if clear_field(input, "parentId") {
        None
    } else {
        input
            .parent_id
            .clone()
            .or_else(|| current.parent_id.clone())
    };
    if parent_id != current.parent_id {
        ensure_parent_valid(conn, workspace_id, issue_id, parent_id.as_deref())?;
    }

    let next_labels: Vec<Label> = match &input.label_names {
        Some(names) => labels::ensure_many(conn, workspace_id, names)?,
        None => current_labels.clone(),
    };

    let priority = input.priority.unwrap_or(current.priority);
    let issue_type = input.issue_type.unwrap_or(current.issue_type);
    let project_id = project.as_ref().map(|value| value.id.clone());

    let mut changed_fields: Vec<&str> = Vec::new();
    if title != current.title {
        changed_fields.push("title");
    }
    if description != current_description {
        changed_fields.push("description");
    }
    if priority != current.priority {
        changed_fields.push("priority");
    }
    if issue_type != current.issue_type {
        changed_fields.push("type");
    }
    if estimate != current.estimate {
        changed_fields.push("estimate");
    }
    if start_date != current.start_date {
        changed_fields.push("startDate");
    }
    if due_date != current.due_date {
        changed_fields.push("dueDate");
    }
    if parent_id != current.parent_id {
        changed_fields.push("parent");
    }
    if project_id != current.project.as_ref().map(|value| value.id.clone()) {
        changed_fields.push("project");
    }

    let completed_at = if status.category == StatusCategory::Completed {
        current.completed_at.clone().or_else(|| Some(now_rfc3339()))
    } else {
        None
    };
    let now = now_rfc3339();

    conn.execute(
        "UPDATE issue SET title = ?1, description = ?2, status_id = ?3, priority = ?4, \
         issue_type = ?5, estimate = ?6, start_date = ?7, due_date = ?8, project_id = ?9, \
         parent_id = ?10, completed_at = ?11, updated_at = ?12 \
         WHERE id = ?13 AND workspace_id = ?14",
        rusqlite::params![
            title,
            description,
            status.id,
            priority.as_str(),
            issue_type.as_str(),
            estimate,
            start_date,
            due_date,
            project.as_ref().map(|value| value.id.as_str()),
            parent_id,
            completed_at,
            now,
            issue_id,
            workspace_id,
        ],
    )?;

    if input.label_names.is_some() {
        labels::set_issue_labels(conn, issue_id, &next_labels)?;
    }

    if status.id != current.status.id {
        activity::record(
            conn,
            issue_id,
            "status_changed",
            &json!({
                "from": current.status.name,
                "to": status.name,
                "fromId": current.status.id,
                "toId": status.id,
            }),
        )?;
    }

    if !changed_fields.is_empty() {
        activity::record(
            conn,
            issue_id,
            "issue_updated",
            &json!({"fields": changed_fields}),
        )?;
    }

    let mut current_label_ids = current_labels
        .iter()
        .map(|label| label.id.clone())
        .collect::<Vec<_>>();
    let mut next_label_ids = next_labels
        .iter()
        .map(|label| label.id.clone())
        .collect::<Vec<_>>();
    current_label_ids.sort();
    next_label_ids.sort();
    if current_label_ids != next_label_ids {
        activity::record(
            conn,
            issue_id,
            "labels_changed",
            &json!({"labels": next_labels.iter().map(|label| &label.name).collect::<Vec<_>>()}),
        )?;
    }

    if enqueue_remote_changes {
        enqueue_linked_github_update(
            conn,
            workspace_id,
            issue_id,
            &LinkedGithubUpdate {
                title: &title,
                description: &description,
                status: &status,
                labels: &next_labels,
                changed_fields: &changed_fields,
                status_changed: status.id != current.status.id,
                labels_changed: current_label_ids != next_label_ids,
                updated_at: &now,
            },
        )?;
    }

    get_detail(conn, workspace_id, issue_id)
}

struct LinkedGithubUpdate<'a> {
    title: &'a str,
    description: &'a str,
    status: &'a crate::domain::issue::StatusSummary,
    labels: &'a [Label],
    changed_fields: &'a [&'a str],
    status_changed: bool,
    labels_changed: bool,
    updated_at: &'a str,
}

fn enqueue_linked_github_update(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
    update: &LinkedGithubUpdate<'_>,
) -> CoreResult<()> {
    let Some(link) = super::sync::get_link(conn, workspace_id, issue_id)? else {
        return Ok(());
    };
    if link.provider != "github" || link.state != "linked" {
        return Ok(());
    }
    let ownership: serde_json::Value =
        serde_json::from_str(&link.field_ownership).unwrap_or_default();
    let owns =
        |field: &str| ownership.get(field).and_then(serde_json::Value::as_str) == Some("remote");
    let mut payload = serde_json::Map::new();
    if update.changed_fields.contains(&"title") && owns("title") {
        payload.insert("title".into(), update.title.into());
    }
    if update.changed_fields.contains(&"description") && owns("body") {
        payload.insert("body".into(), update.description.into());
    }
    if update.status_changed && owns("status") {
        let state = if update.status.category.is_closed() {
            "closed"
        } else {
            "open"
        };
        payload.insert("state".into(), state.into());
    }
    if update.labels_changed && owns("labels") {
        payload.insert(
            "labels".into(),
            serde_json::json!(update
                .labels
                .iter()
                .map(|label| label.name.as_str())
                .collect::<Vec<_>>()),
        );
    }
    if payload.is_empty() {
        return Ok(());
    }
    let payload_json = serde_json::Value::Object(payload).to_string();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    payload_json.hash(&mut hasher);
    let fingerprint = format!("{:016x}", hasher.finish());
    let idempotency_key = format!(
        "github:{}:{}:{fingerprint}",
        link.remote_id, update.updated_at
    );
    super::sync::enqueue_outbox(
        conn,
        workspace_id,
        &crate::domain::sync::OutboxInput {
            local_issue_id: Some(issue_id.to_owned()),
            operation: "update_issue".into(),
            payload_json,
            idempotency_key,
            fingerprint,
            reconciliation_hint: Some(format!(
                "{}/issues/{}",
                link.remote_repository_key.as_deref().unwrap_or_default(),
                link.remote_number
            )),
        },
    )?;
    Ok(())
}

pub fn archive(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
    archived: bool,
) -> CoreResult<IssueDetail> {
    let current = get_summary(conn, workspace_id, issue_id)?;
    let already = current.archived_at.is_some();
    if already != archived {
        let now = now_rfc3339();
        conn.execute(
            "UPDATE issue SET archived_at = ?1, updated_at = ?2 \
             WHERE id = ?3 AND workspace_id = ?4",
            rusqlite::params![
                if archived { Some(now) } else { None },
                now_rfc3339(),
                issue_id,
                workspace_id
            ],
        )?;
        activity::record(
            conn,
            issue_id,
            if archived { "archived" } else { "unarchived" },
            &json!({"archived": archived}),
        )?;
    }
    get_detail(conn, workspace_id, issue_id)
}

pub fn duplicate(conn: &Connection, workspace_id: &str, issue_id: &str) -> CoreResult<IssueDetail> {
    let source = get_detail(conn, workspace_id, issue_id)?;
    let title: String = format!("{} (copy)", source.summary.title)
        .chars()
        .take(crate::domain::TITLE_MAX)
        .collect();
    let labels = source
        .summary
        .labels
        .iter()
        .map(|label| label.name.clone())
        .collect::<Vec<_>>();

    let input = CreateIssueInput {
        title,
        description: source.description.clone(),
        status_id: Some(source.summary.status.id.clone()),
        priority: source.summary.priority,
        issue_type: source.summary.issue_type,
        estimate: source.summary.estimate,
        start_date: source.summary.start_date.clone(),
        due_date: source.summary.due_date.clone(),
        project_id: source
            .summary
            .project
            .as_ref()
            .map(|value| value.id.clone()),
        parent_id: None,
        label_names: labels,
    };
    let new_id = create(conn, workspace_id, &input)?;
    relations::add(
        conn,
        workspace_id,
        issue_id,
        &new_id,
        RelationInputKind::Related,
    )?;
    activity::record(conn, issue_id, "duplicated", &json!({"newId": new_id}))?;
    get_detail(conn, workspace_id, &new_id)
}

pub fn batch_update(
    conn: &Connection,
    workspace_id: &str,
    issue_ids: &[String],
    input: &BatchIssueInput,
) -> CoreResult<Vec<IssueDetail>> {
    if issue_ids.is_empty() {
        return Err(CoreError::validation("ids", "Select at least one issue."));
    }
    if issue_ids.len() > 200 {
        return Err(CoreError::validation(
            "ids",
            "At most 200 issues can be changed at once.",
        ));
    }

    let mut details = Vec::new();
    for issue_id in issue_ids {
        let patch = UpdateIssueInput {
            status_id: input.status_id.clone(),
            priority: input.priority,
            issue_type: input.issue_type,
            ..Default::default()
        };
        let detail = if patch.status_id.is_some()
            || patch.priority.is_some()
            || patch.issue_type.is_some()
        {
            update(conn, workspace_id, issue_id, &patch)?
        } else {
            get_detail(conn, workspace_id, issue_id)?
        };

        if !input.add_label_names.is_empty() || !input.remove_label_names.is_empty() {
            if !input.add_label_names.is_empty() {
                labels::add_to_issue(conn, workspace_id, issue_id, &input.add_label_names)?;
            }
            if !input.remove_label_names.is_empty() {
                labels::remove_from_issue(conn, workspace_id, issue_id, &input.remove_label_names)?;
            }
            activity::record(
                conn,
                issue_id,
                "labels_changed",
                &json!({
                    "added": input.add_label_names,
                    "removed": input.remove_label_names,
                }),
            )?;
        }

        let detail = if input.add_label_names.is_empty() && input.remove_label_names.is_empty() {
            detail
        } else {
            get_detail(conn, workspace_id, issue_id)?
        };
        if !input.add_label_names.is_empty() || !input.remove_label_names.is_empty() {
            enqueue_linked_github_update(
                conn,
                workspace_id,
                issue_id,
                &LinkedGithubUpdate {
                    title: &detail.summary.title,
                    description: &detail.description,
                    status: &detail.summary.status,
                    labels: &detail.summary.labels,
                    changed_fields: &[],
                    status_changed: false,
                    labels_changed: true,
                    updated_at: &detail.summary.updated_at,
                },
            )?;
        }
        let detail = match input.archive {
            Some(archived) => archive(conn, workspace_id, issue_id, archived)?,
            None => detail,
        };
        details.push(detail);
    }
    Ok(details)
}

pub fn add_comment(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
    body: &str,
) -> CoreResult<IssueDetail> {
    ensure_issue_in_workspace(conn, workspace_id, issue_id)?;
    comments::add(conn, issue_id, body)?;
    activity::record(conn, issue_id, "comment_added", &json!({}))?;
    get_detail(conn, workspace_id, issue_id)
}

pub fn add_relation(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
    related_issue_id: &str,
    kind: RelationInputKind,
) -> CoreResult<IssueDetail> {
    ensure_issue_in_workspace(conn, workspace_id, issue_id)?;
    let relation = relations::add(conn, workspace_id, issue_id, related_issue_id, kind)?;
    activity::record(
        conn,
        issue_id,
        "relation_added",
        &json!({"kind": relation.kind, "issueKey": relation.issue.key}),
    )?;
    get_detail(conn, workspace_id, issue_id)
}

pub fn remove_relation(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
    relation_id: &str,
) -> CoreResult<IssueDetail> {
    ensure_issue_in_workspace(conn, workspace_id, issue_id)?;
    relations::remove(conn, workspace_id, relation_id)?;
    activity::record(conn, issue_id, "relation_removed", &json!({}))?;
    get_detail(conn, workspace_id, issue_id)
}

pub fn list_activity(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
) -> CoreResult<Vec<ActivityEvent>> {
    ensure_issue_in_workspace(conn, workspace_id, issue_id)?;
    activity::list(conn, issue_id)
}

pub fn list_relations(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
) -> CoreResult<Vec<IssueRelation>> {
    ensure_issue_in_workspace(conn, workspace_id, issue_id)?;
    relations::list_for_issue(conn, issue_id)
}

fn ensure_issue_in_workspace(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
) -> CoreResult<()> {
    let count: i64 = conn.query_row(
        "SELECT count(*) FROM issue WHERE id = ?1 AND workspace_id = ?2 AND deleted_at IS NULL",
        rusqlite::params![issue_id, workspace_id],
        |row| row.get(0),
    )?;
    if count == 0 {
        return Err(CoreError::not_found("Issue"));
    }
    Ok(())
}

fn ensure_parent_valid(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
    parent_id: Option<&str>,
) -> CoreResult<()> {
    let Some(parent_id) = parent_id else {
        return Ok(());
    };
    if parent_id == issue_id {
        return Err(CoreError::validation(
            "parentId",
            "An issue cannot be its own parent.",
        ));
    }

    let mut seen = HashSet::new();
    let mut current = Some(parent_id.to_owned());
    while let Some(candidate) = current {
        if candidate == issue_id {
            return Err(CoreError::validation(
                "parentId",
                "This parent would create a cycle in the issue hierarchy.",
            ));
        }
        if !seen.insert(candidate.clone()) {
            return Err(CoreError::validation(
                "parentId",
                "The issue hierarchy contains a cycle.",
            ));
        }
        let row: Option<(String, Option<String>)> = conn
            .query_row(
                "SELECT workspace_id, parent_id FROM issue \
                 WHERE id = ?1 AND deleted_at IS NULL",
                [&candidate],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let Some((parent_workspace, next_parent)) = row else {
            return Err(CoreError::validation(
                "parentId",
                "Parent issue was not found.",
            ));
        };
        if parent_workspace != workspace_id {
            return Err(CoreError::validation(
                "parentId",
                "Parent issue belongs to another workspace.",
            ));
        }
        current = next_parent;
    }
    Ok(())
}

fn clear_field(input: &UpdateIssueInput, field: &str) -> bool {
    input.clear_fields.iter().any(|value| value == field)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::domain::{FilterField, FilterNode, FilterOperator, LogicalOp, SortSpec};

    fn fixture() -> (tempfile::TempDir, Database, String) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("open");
        let workspace = db
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "Issues"))
            .expect("workspace");
        (dir, db, workspace.id)
    }

    fn issue_input(title: &str) -> CreateIssueInput {
        CreateIssueInput {
            title: title.into(),
            description: format!("{title} description"),
            status_id: None,
            priority: Priority::Medium,
            issue_type: IssueType::Task,
            estimate: Some(2.0),
            start_date: None,
            due_date: None,
            project_id: None,
            parent_id: None,
            label_names: vec!["capture".into()],
        }
    }

    #[test]
    fn creates_lists_updates_and_archives_issues() {
        let (_dir, db, workspace_id) = fixture();
        let (first_id, second_id) = db
            .with_conn(|conn| {
                let first = create(conn, &workspace_id, &issue_input("First issue"))?;
                let second = create(conn, &workspace_id, &issue_input("Second issue"))?;
                Ok((first, second))
            })
            .expect("create issues");

        let first = db
            .with_conn(|conn| get_detail(conn, &workspace_id, &first_id))
            .expect("detail");
        assert_eq!(first.summary.key, "FD-1");
        assert_eq!(first.summary.status.name, "Backlog");
        assert_eq!(first.summary.labels.len(), 1);
        assert_eq!(first.activity[0].kind, "issue_created");

        let listed = db
            .with_conn(|conn| {
                list(
                    conn,
                    &workspace_id,
                    &IssueQuery {
                        filter: None,
                        sort: vec![],
                        search: None,
                        include_archived: false,
                        limit: None,
                        offset: None,
                    },
                )
            })
            .expect("list");
        assert_eq!(listed.len(), 2);

        let updated = db
            .with_conn(|conn| {
                update(
                    conn,
                    &workspace_id,
                    &first_id,
                    &UpdateIssueInput {
                        priority: Some(Priority::Urgent),
                        label_names: Some(vec!["capture".into(), "regression".into()]),
                        ..Default::default()
                    },
                )
            })
            .expect("update");
        assert_eq!(updated.summary.priority, Priority::Urgent);
        assert_eq!(updated.summary.labels.len(), 2);

        let archived = db
            .with_conn(|conn| archive(conn, &workspace_id, &first_id, true))
            .expect("archive");
        assert!(archived.summary.archived_at.is_some());

        let listed = db
            .with_conn(|conn| {
                list(
                    conn,
                    &workspace_id,
                    &IssueQuery {
                        filter: None,
                        sort: vec![],
                        search: None,
                        include_archived: false,
                        limit: None,
                        offset: None,
                    },
                )
            })
            .expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, second_id);
    }

    #[test]
    fn hierarchy_cycle_is_rejected() {
        let (_dir, db, workspace_id) = fixture();
        let (parent_id, child_id) = db
            .with_conn(|conn| {
                let parent = create(conn, &workspace_id, &issue_input("Parent"))?;
                let mut input = issue_input("Child");
                input.parent_id = Some(parent.clone());
                let child = create(conn, &workspace_id, &input)?;
                Ok((parent, child))
            })
            .expect("hierarchy");

        let error = db
            .with_conn(|conn| {
                update(
                    conn,
                    &workspace_id,
                    &parent_id,
                    &UpdateIssueInput {
                        parent_id: Some(child_id),
                        ..Default::default()
                    },
                )
            })
            .expect_err("cycle must fail");
        assert_eq!(error.category(), crate::error::ErrorCategory::Validation);
    }

    #[test]
    fn filters_and_search_are_parameterized() {
        let (_dir, db, workspace_id) = fixture();
        db.with_conn(|conn| {
            let mut high = issue_input("High capture bug");
            high.priority = Priority::High;
            high.issue_type = IssueType::Bug;
            create(conn, &workspace_id, &high)?;
            let mut low = issue_input("Low documentation task");
            low.priority = Priority::Low;
            low.issue_type = IssueType::Task;
            create(conn, &workspace_id, &low)?;
            Ok(())
        })
        .expect("seed");

        let query = IssueQuery {
            filter: Some(FilterNode::Group {
                op: LogicalOp::And,
                conditions: vec![
                    FilterNode::Group {
                        op: LogicalOp::Or,
                        conditions: vec![
                            FilterNode::Condition {
                                field: FilterField::Priority,
                                operator: FilterOperator::Eq,
                                value: serde_json::json!("high"),
                            },
                            FilterNode::Condition {
                                field: FilterField::Priority,
                                operator: FilterOperator::Eq,
                                value: serde_json::json!("urgent"),
                            },
                        ],
                    },
                    FilterNode::Condition {
                        field: FilterField::Type,
                        operator: FilterOperator::Eq,
                        value: serde_json::json!("bug"),
                    },
                ],
            }),
            sort: vec![SortSpec {
                field: crate::domain::SortField::UpdatedAt,
                direction: crate::domain::SortDirection::Desc,
            }],
            search: None,
            include_archived: false,
            limit: None,
            offset: None,
        };
        let results = db
            .with_conn(|conn| list(conn, &workspace_id, &query))
            .expect("filtered list");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "High capture bug");

        let search = IssueQuery {
            filter: None,
            sort: vec![],
            search: Some("documentation".into()),
            include_archived: false,
            limit: None,
            offset: None,
        };
        let results = db
            .with_conn(|conn| list(conn, &workspace_id, &search))
            .expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Low documentation task");
    }

    #[test]
    fn batch_relations_and_comments() {
        let (_dir, db, workspace_id) = fixture();
        let (first_id, second_id) = db
            .with_conn(|conn| {
                let first = create(conn, &workspace_id, &issue_input("First"))?;
                let second = create(conn, &workspace_id, &issue_input("Second"))?;
                Ok((first, second))
            })
            .expect("create");

        let with_relation = db
            .with_conn(|conn| {
                add_relation(
                    conn,
                    &workspace_id,
                    &first_id,
                    &second_id,
                    RelationInputKind::Blocks,
                )
            })
            .expect("relation");
        assert_eq!(with_relation.relations.len(), 1);
        assert_eq!(
            with_relation.relations[0].kind,
            crate::domain::RelationKind::Blocks
        );

        let commented = db
            .with_conn(|conn| add_comment(conn, &workspace_id, &first_id, "Working on it"))
            .expect("comment");
        assert_eq!(commented.comments.len(), 1);

        let batch = db
            .with_conn(|conn| {
                batch_update(
                    conn,
                    &workspace_id,
                    &[first_id.clone(), second_id.clone()],
                    &BatchIssueInput {
                        priority: Some(Priority::Urgent),
                        ..Default::default()
                    },
                )
            })
            .expect("batch");
        assert_eq!(batch.len(), 2);
        assert!(batch
            .iter()
            .all(|detail| detail.summary.priority == Priority::Urgent));
    }

    #[test]
    fn label_project_clear_and_injection_filters() {
        let (_dir, db, workspace_id) = fixture();
        let project = db
            .with_conn(|conn| {
                crate::db::repositories::projects::create(conn, &workspace_id, "Filter Project")
            })
            .expect("project");

        let (first_id, second_id) = db
            .with_conn(|conn| {
                let mut first = issue_input("Labeled capture bug");
                first.project_id = Some(project.id.clone());
                first.due_date = Some("2026-12-01".into());
                let first = create(conn, &workspace_id, &first)?;
                let mut second = issue_input("Unlabeled documentation");
                second.label_names = vec![];
                let second = create(conn, &workspace_id, &second)?;
                Ok((first, second))
            })
            .expect("seed");

        let label_query = IssueQuery {
            filter: Some(FilterNode::Condition {
                field: FilterField::Label,
                operator: FilterOperator::IsEmpty,
                value: serde_json::Value::Null,
            }),
            sort: vec![],
            search: None,
            include_archived: false,
            limit: None,
            offset: None,
        };
        let results = db
            .with_conn(|conn| list(conn, &workspace_id, &label_query))
            .expect("label filter");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, second_id);

        let project_query = IssueQuery {
            filter: Some(FilterNode::Condition {
                field: FilterField::Project,
                operator: FilterOperator::Eq,
                value: serde_json::json!(project.id),
            }),
            sort: vec![],
            search: None,
            include_archived: false,
            limit: None,
            offset: None,
        };
        let results = db
            .with_conn(|conn| list(conn, &workspace_id, &project_query))
            .expect("project filter");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, first_id);

        let injection_query = IssueQuery {
            filter: Some(FilterNode::Condition {
                field: FilterField::Title,
                operator: FilterOperator::Contains,
                value: serde_json::json!("' OR 1=1 --"),
            }),
            sort: vec![],
            search: None,
            include_archived: false,
            limit: None,
            offset: None,
        };
        assert!(db
            .with_conn(|conn| list(conn, &workspace_id, &injection_query))
            .expect("injection filter")
            .is_empty());

        let detail = db
            .with_conn(|conn| {
                update(
                    conn,
                    &workspace_id,
                    &first_id,
                    &UpdateIssueInput {
                        clear_fields: vec!["dueDate".into()],
                        ..Default::default()
                    },
                )
            })
            .expect("clear due date");
        assert!(detail.summary.due_date.is_none());
        assert!(detail
            .activity
            .iter()
            .any(|event| event.kind == "issue_updated"));

        let renamed = db
            .with_conn(|conn| {
                update(
                    conn,
                    &workspace_id,
                    &first_id,
                    &UpdateIssueInput {
                        title: Some("Findable needle".into()),
                        ..Default::default()
                    },
                )
            })
            .expect("rename");
        assert_eq!(renamed.summary.title, "Findable needle");

        let search = IssueQuery {
            filter: None,
            sort: vec![],
            search: Some("needle".into()),
            include_archived: false,
            limit: None,
            offset: None,
        };
        let results = db
            .with_conn(|conn| list(conn, &workspace_id, &search))
            .expect("fts after rename");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, first_id);
    }

    #[test]
    fn duplicates_issue_with_new_key() {
        let (_dir, db, workspace_id) = fixture();
        let id = db
            .with_conn(|conn| create(conn, &workspace_id, &issue_input("Original")))
            .expect("create");
        let copy = db
            .with_conn(|conn| duplicate(conn, &workspace_id, &id))
            .expect("duplicate");
        assert_ne!(copy.summary.id, id);
        assert_eq!(copy.summary.key, "FD-2");
        assert!(copy.summary.title.ends_with("(copy)"));
    }
}
