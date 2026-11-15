use rusqlite::{Connection, OptionalExtension};
use serde_json::Value;

use crate::domain::inbox::{InboxEvent, InboxFilter};
use crate::domain::time::now_rfc3339;
use crate::error::{CoreError, CoreResult};

pub fn list(
    conn: &Connection,
    workspace_id: &str,
    filter: InboxFilter,
) -> CoreResult<Vec<InboxEvent>> {
    let mut statement = conn.prepare(
        "SELECT a.id, a.kind, a.data, a.created_at, i.id, i.key, i.title, i.source, \
                s.read_at, s.archived_at \
         FROM activity_event a \
         JOIN issue i ON i.id = a.issue_id \
         LEFT JOIN inbox_state s ON s.event_id = a.id \
         WHERE i.workspace_id = ?1 AND i.deleted_at IS NULL AND s.archived_at IS NULL \
         ORDER BY a.created_at DESC, a.rowid DESC LIMIT 300",
    )?;
    let rows = statement.query_map([workspace_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, Option<String>>(8)?,
            row.get::<_, Option<String>>(9)?,
        ))
    })?;

    let mut events = Vec::new();
    for row in rows {
        let (id, kind, raw, created_at, issue_id, key, issue_title, source, read_at, archived_at) =
            row?;
        let data: Value = serde_json::from_str(&raw).unwrap_or(Value::Null);
        let (title, summary) = describe(&kind, &data, &issue_title);
        let source = if source == "github" {
            "github"
        } else {
            "local"
        };
        let event = InboxEvent {
            id,
            source: source.into(),
            kind,
            title,
            summary,
            entity_type: "issue".into(),
            entity_id: issue_id,
            entity_key: key,
            entity_title: issue_title,
            created_at,
            read: read_at.is_some(),
            archived: archived_at.is_some(),
        };
        if matches!(filter, InboxFilter::All)
            || (filter == InboxFilter::Unread && !event.read)
            || (filter == InboxFilter::Github && event.source == "github")
            || (filter == InboxFilter::Local && event.source == "local")
            || (filter == InboxFilter::Sync && event.source == "sync")
        {
            events.push(event);
        }
    }
    Ok(events)
}

pub fn mark_read(
    conn: &Connection,
    workspace_id: &str,
    event_id: &str,
    read: bool,
) -> CoreResult<()> {
    ensure_event(conn, workspace_id, event_id)?;
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO inbox_state (event_id, read_at, archived_at) VALUES (?1, ?2, NULL) \
         ON CONFLICT(event_id) DO UPDATE SET read_at = excluded.read_at",
        rusqlite::params![event_id, if read { Some(now) } else { None }],
    )?;
    Ok(())
}

pub fn archive(conn: &Connection, workspace_id: &str, event_id: &str) -> CoreResult<()> {
    ensure_event(conn, workspace_id, event_id)?;
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO inbox_state (event_id, read_at, archived_at) VALUES (?1, ?2, ?2) \
         ON CONFLICT(event_id) DO UPDATE SET read_at = coalesce(inbox_state.read_at, excluded.read_at), \
          archived_at = excluded.archived_at",
        rusqlite::params![event_id, now],
    )?;
    Ok(())
}

pub fn mark_all_read(conn: &Connection, workspace_id: &str) -> CoreResult<i64> {
    let count: i64 = conn.query_row(
        "SELECT count(*) FROM activity_event a JOIN issue i ON i.id = a.issue_id \
         LEFT JOIN inbox_state s ON s.event_id = a.id \
         WHERE i.workspace_id = ?1 AND i.deleted_at IS NULL \
           AND s.archived_at IS NULL AND s.read_at IS NULL",
        [workspace_id],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO inbox_state (event_id, read_at, archived_at) \
         SELECT a.id, ?2, NULL FROM activity_event a JOIN issue i ON i.id = a.issue_id \
         LEFT JOIN inbox_state s ON s.event_id = a.id \
         WHERE i.workspace_id = ?1 AND i.deleted_at IS NULL \
           AND s.archived_at IS NULL AND s.read_at IS NULL \
         ON CONFLICT(event_id) DO UPDATE SET read_at = excluded.read_at",
        rusqlite::params![workspace_id, now_rfc3339()],
    )?;
    Ok(count)
}

fn ensure_event(conn: &Connection, workspace_id: &str, event_id: &str) -> CoreResult<()> {
    let exists: Option<bool> = conn
        .query_row(
            "SELECT 1 FROM activity_event a JOIN issue i ON i.id = a.issue_id \
             WHERE a.id = ?1 AND i.workspace_id = ?2",
            rusqlite::params![event_id, workspace_id],
            |_| Ok(true),
        )
        .optional()?;
    if exists != Some(true) {
        return Err(CoreError::not_found("Inbox event"));
    }
    Ok(())
}

fn describe(kind: &str, data: &Value, issue_title: &str) -> (String, String) {
    let text = |key: &str| data.get(key).and_then(Value::as_str).unwrap_or("");
    match kind {
        "issue_created" => (
            "Issue created".into(),
            if text("title").is_empty() {
                issue_title.into()
            } else {
                text("title").into()
            },
        ),
        "status_changed" => (
            "Status changed".into(),
            format!("{} → {}", text("from"), text("to")),
        ),
        "issue_updated" => {
            let fields = data
                .get("fields")
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            ("Issue updated".into(), fields)
        }
        "labels_changed" => ("Labels changed".into(), String::new()),
        "comment_added" => ("Comment added".into(), String::new()),
        "archived" => ("Issue archived".into(), issue_title.into()),
        "unarchived" => ("Issue restored".into(), issue_title.into()),
        "relation_added" => ("Relation added".into(), String::new()),
        "relation_removed" => ("Relation removed".into(), String::new()),
        "duplicated" => ("Issue duplicated".into(), String::new()),
        other => (other.replace('_', " "), issue_title.into()),
    }
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
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "Inbox"))
            .expect("workspace");
        let issue = db
            .with_conn(|conn| {
                crate::db::repositories::issues::create(
                    conn,
                    &workspace.id,
                    &CreateIssueInput {
                        title: "Inbox issue".into(),
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
    fn lists_and_marks_events_read() {
        let (_dir, db, workspace_id, issue_id) = fixture();
        db.with_conn(|conn| {
            crate::db::repositories::issues::add_comment(conn, &workspace_id, &issue_id, "hello")
        })
        .expect("comment");
        let events = db
            .with_conn(|conn| list(conn, &workspace_id, InboxFilter::All))
            .expect("list");
        assert!(events.len() >= 2);
        assert!(!events[0].read);
        let unread = db
            .with_conn(|conn| list(conn, &workspace_id, InboxFilter::Unread))
            .expect("unread");
        assert_eq!(unread.len(), events.len());

        db.with_conn(|conn| mark_read(conn, &workspace_id, &events[0].id, true))
            .expect("read");
        let unread = db
            .with_conn(|conn| list(conn, &workspace_id, InboxFilter::Unread))
            .expect("unread");
        assert_eq!(unread.len(), events.len() - 1);

        db.with_conn(|conn| archive(conn, &workspace_id, &events[0].id))
            .expect("archive");
        let all = db
            .with_conn(|conn| list(conn, &workspace_id, InboxFilter::All))
            .expect("list");
        assert!(!all.iter().any(|event| event.id == events[0].id));
    }

    #[test]
    fn mark_all_read_is_idempotent() {
        let (_dir, db, workspace_id, _issue_id) = fixture();
        let first = db
            .with_conn(|conn| mark_all_read(conn, &workspace_id))
            .expect("first");
        assert!(first >= 1);
        let second = db
            .with_conn(|conn| mark_all_read(conn, &workspace_id))
            .expect("second");
        assert_eq!(second, 0);
    }
}
