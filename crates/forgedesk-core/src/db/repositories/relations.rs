use rusqlite::{Connection, OptionalExtension};

use crate::domain::time::now_rfc3339;
use crate::domain::{IssueRelation, RelatedIssue, RelationInputKind, RelationKind};
use crate::error::{CoreError, CoreResult};

use super::workflow::parse_status;

pub fn add(
    conn: &Connection,
    workspace_id: &str,
    from_issue_id: &str,
    to_issue_id: &str,
    kind: RelationInputKind,
) -> CoreResult<IssueRelation> {
    if from_issue_id == to_issue_id {
        return Err(CoreError::validation(
            "relatedIssueId",
            "An issue cannot be related to itself.",
        ));
    }

    let from_workspace: String = conn
        .query_row(
            "SELECT workspace_id FROM issue WHERE id = ?1 AND deleted_at IS NULL",
            [from_issue_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| CoreError::not_found("Issue"))?;
    let to_workspace: String = conn
        .query_row(
            "SELECT workspace_id FROM issue WHERE id = ?1 AND deleted_at IS NULL",
            [to_issue_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| CoreError::not_found("Issue"))?;
    if from_workspace != workspace_id || to_workspace != workspace_id {
        return Err(CoreError::not_found("Issue"));
    }

    let (from, to) = if kind == RelationInputKind::Related && from_issue_id > to_issue_id {
        (to_issue_id, from_issue_id)
    } else {
        (from_issue_id, to_issue_id)
    };
    let relation_type = match kind {
        RelationInputKind::Blocks => "blocks",
        RelationInputKind::Related => "related",
        RelationInputKind::Duplicate => "duplicate",
    };

    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM issue_relation \
             WHERE from_issue_id = ?1 AND to_issue_id = ?2 AND relation_type = ?3",
            rusqlite::params![from, to, relation_type],
            |row| row.get(0),
        )
        .optional()?;

    let id = if let Some(id) = existing {
        id
    } else {
        let id = uuid::Uuid::now_v7().to_string();
        conn.execute(
            "INSERT INTO issue_relation \
             (id, from_issue_id, to_issue_id, relation_type, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, from, to, relation_type, now_rfc3339()],
        )?;
        id
    };

    relation_view(conn, &id, from_issue_id)?.ok_or_else(|| CoreError::not_found("Relation"))
}

pub fn remove(conn: &Connection, workspace_id: &str, relation_id: &str) -> CoreResult<()> {
    let removed = conn.execute(
        "DELETE FROM issue_relation WHERE id = ?1 AND EXISTS \
         (SELECT 1 FROM issue i WHERE i.id = from_issue_id AND i.workspace_id = ?2)",
        rusqlite::params![relation_id, workspace_id],
    )?;
    if removed == 0 {
        return Err(CoreError::not_found("Relation"));
    }
    Ok(())
}

pub fn list_for_issue(conn: &Connection, issue_id: &str) -> CoreResult<Vec<IssueRelation>> {
    let mut statement = conn.prepare(
        "SELECT r.id, r.relation_type, r.from_issue_id, r.to_issue_id, \
                i.id, i.key, i.title, s.id, s.name, s.category, s.color, s.position, s.system_key \
         FROM issue_relation r \
         JOIN issue i ON i.id = CASE \
             WHEN r.from_issue_id = ?1 THEN r.to_issue_id ELSE r.from_issue_id END \
         JOIN workflow_status s ON s.id = i.status_id \
         WHERE r.from_issue_id = ?1 OR r.to_issue_id = ?1 \
         ORDER BY r.created_at ASC, r.rowid ASC",
    )?;
    let rows = statement.query_map([issue_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, String>(9)?,
            row.get::<_, String>(10)?,
            row.get::<_, i64>(11)?,
            row.get::<_, Option<String>>(12)?,
        ))
    })?;

    let mut relations = Vec::new();
    for row in rows {
        let (
            id,
            relation_type,
            from_id,
            _to_id,
            other_id,
            other_key,
            other_title,
            status_id,
            status_name,
            category,
            color,
            position,
            system_key,
        ) = row?;
        relations.push(IssueRelation {
            id,
            kind: map_kind(&relation_type, from_id == issue_id),
            issue: RelatedIssue {
                id: other_id,
                key: other_key,
                title: other_title,
                status: parse_status(
                    status_id,
                    status_name,
                    &category,
                    &color,
                    position,
                    system_key,
                )?,
            },
        });
    }
    Ok(relations)
}

fn relation_view(
    conn: &Connection,
    relation_id: &str,
    perspective_issue_id: &str,
) -> CoreResult<Option<IssueRelation>> {
    let mut statement = conn.prepare(
        "SELECT r.id, r.relation_type, r.from_issue_id, \
                i.id, i.key, i.title, s.id, s.name, s.category, s.color, s.position, s.system_key \
         FROM issue_relation r \
         JOIN issue i ON i.id = CASE \
             WHEN r.from_issue_id = ?1 THEN r.to_issue_id ELSE r.from_issue_id END \
         JOIN workflow_status s ON s.id = i.status_id \
         WHERE r.id = ?2",
    )?;
    let row = statement
        .query_row(
            rusqlite::params![perspective_issue_id, relation_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, i64>(10)?,
                    row.get::<_, Option<String>>(11)?,
                ))
            },
        )
        .optional()?;

    let Some((
        id,
        relation_type,
        from_id,
        other_id,
        other_key,
        other_title,
        status_id,
        status_name,
        category,
        color,
        position,
        system_key,
    )) = row
    else {
        return Ok(None);
    };

    Ok(Some(IssueRelation {
        id,
        kind: map_kind(&relation_type, from_id == perspective_issue_id),
        issue: RelatedIssue {
            id: other_id,
            key: other_key,
            title: other_title,
            status: parse_status(
                status_id,
                status_name,
                &category,
                &color,
                position,
                system_key,
            )?,
        },
    }))
}

fn map_kind(relation_type: &str, perspective_is_from: bool) -> RelationKind {
    match (relation_type, perspective_is_from) {
        ("blocks", true) => RelationKind::Blocks,
        ("blocks", false) => RelationKind::BlockedBy,
        ("duplicate", true) => RelationKind::Duplicate,
        ("duplicate", false) => RelationKind::DuplicatedBy,
        _ => RelationKind::Related,
    }
}
