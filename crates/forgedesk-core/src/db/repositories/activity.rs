use rusqlite::Connection;
use serde_json::Value;

use crate::domain::time::now_rfc3339;
use crate::domain::ActivityEvent;
use crate::error::{CoreError, CoreResult};

pub fn record(conn: &Connection, issue_id: &str, kind: &str, data: &Value) -> CoreResult<()> {
    let payload = serde_json::to_string(data)
        .map_err(|error| CoreError::internal(format!("Activity payload failed: {error}")))?;
    conn.execute(
        "INSERT INTO activity_event (id, issue_id, kind, data, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            uuid::Uuid::now_v7().to_string(),
            issue_id,
            kind,
            payload,
            now_rfc3339()
        ],
    )?;
    Ok(())
}

pub fn list(conn: &Connection, issue_id: &str) -> CoreResult<Vec<ActivityEvent>> {
    let mut statement = conn.prepare(
        "SELECT id, issue_id, kind, data, created_at FROM activity_event \
         WHERE issue_id = ?1 ORDER BY created_at ASC, rowid ASC LIMIT 200",
    )?;
    let rows = statement.query_map([issue_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;

    let mut events = Vec::new();
    for row in rows {
        let (id, issue_id, kind, raw, created_at) = row?;
        events.push(ActivityEvent {
            id,
            issue_id,
            kind,
            data: serde_json::from_str(&raw).unwrap_or(Value::Null),
            created_at,
        });
    }
    Ok(events)
}
