use rusqlite::{Connection, OptionalExtension};

use crate::domain::time::now_rfc3339;
use crate::domain::{validate_label_name, Label, StatusColor};
use crate::error::CoreResult;

pub fn list(conn: &Connection, workspace_id: &str) -> CoreResult<Vec<Label>> {
    let mut statement = conn.prepare(
        "SELECT id, name, color FROM label WHERE workspace_id = ?1 \
         ORDER BY name COLLATE NOCASE ASC",
    )?;
    let rows = statement.query_map([workspace_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let mut labels = Vec::new();
    for row in rows {
        let (id, name, color) = row?;
        labels.push(Label {
            id,
            name,
            color: StatusColor::parse(&color)?,
        });
    }
    Ok(labels)
}

/// Resolve label names case-insensitively, creating missing labels as neutral.
pub fn ensure_many(
    conn: &Connection,
    workspace_id: &str,
    names: &[String],
) -> CoreResult<Vec<Label>> {
    let mut unique: Vec<String> = Vec::new();
    for raw in names {
        let name = validate_label_name(raw)?;
        if !unique
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(&name))
        {
            unique.push(name);
        }
    }

    let mut labels = Vec::new();
    for name in unique {
        let existing = conn
            .query_row(
                "SELECT id, name, color FROM label \
                 WHERE workspace_id = ?1 AND lower(name) = lower(?2)",
                rusqlite::params![workspace_id, name],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?;

        let (id, stored_name, color) = if let Some(existing) = existing {
            existing
        } else {
            let id = uuid::Uuid::now_v7().to_string();
            conn.execute(
                "INSERT INTO label (id, workspace_id, name, color, created_at) \
                 VALUES (?1, ?2, ?3, 'neutral', ?4)",
                rusqlite::params![id, workspace_id, name, now_rfc3339()],
            )?;
            (id, name, "neutral".to_owned())
        };

        labels.push(Label {
            id,
            name: stored_name,
            color: StatusColor::parse(&color)?,
        });
    }
    Ok(labels)
}

pub fn set_issue_labels(conn: &Connection, issue_id: &str, labels: &[Label]) -> CoreResult<()> {
    conn.execute("DELETE FROM issue_label WHERE issue_id = ?1", [issue_id])?;
    for label in labels {
        conn.execute(
            "INSERT OR IGNORE INTO issue_label (issue_id, label_id) VALUES (?1, ?2)",
            rusqlite::params![issue_id, label.id],
        )?;
    }
    Ok(())
}

pub fn add_to_issue(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
    names: &[String],
) -> CoreResult<Vec<Label>> {
    let labels = ensure_many(conn, workspace_id, names)?;
    for label in &labels {
        conn.execute(
            "INSERT OR IGNORE INTO issue_label (issue_id, label_id) VALUES (?1, ?2)",
            rusqlite::params![issue_id, label.id],
        )?;
    }
    Ok(labels)
}

pub fn remove_from_issue(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
    names: &[String],
) -> CoreResult<()> {
    for raw in names {
        let name = validate_label_name(raw)?;
        conn.execute(
            "DELETE FROM issue_label WHERE issue_id = ?1 AND label_id IN \
             (SELECT id FROM label WHERE workspace_id = ?2 AND lower(name) = lower(?3))",
            rusqlite::params![issue_id, workspace_id, name],
        )?;
    }
    Ok(())
}

pub fn labels_for_issues(
    conn: &Connection,
    issue_ids: &[String],
) -> CoreResult<std::collections::HashMap<String, Vec<Label>>> {
    let mut map: std::collections::HashMap<String, Vec<Label>> = std::collections::HashMap::new();
    if issue_ids.is_empty() {
        return Ok(map);
    }

    let placeholders = vec!["?"; issue_ids.len()].join(", ");
    let sql = format!(
        "SELECT il.issue_id, l.id, l.name, l.color FROM issue_label il \
         JOIN label l ON l.id = il.label_id WHERE il.issue_id IN ({placeholders}) \
         ORDER BY l.name COLLATE NOCASE ASC"
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(rusqlite::params_from_iter(issue_ids.iter()), |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    for row in rows {
        let (issue_id, id, name, color) = row?;
        map.entry(issue_id).or_default().push(Label {
            id,
            name,
            color: StatusColor::parse(&color)?,
        });
    }
    Ok(map)
}
