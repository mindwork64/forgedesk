use rusqlite::{Connection, OptionalExtension, Row};

use crate::domain::sync::{
    OutboxInput, OutboxJob, OutboxState, RemoteIssueLink, SyncConflict, SyncCursor, SyncCursorInput,
};
use crate::domain::time::now_rfc3339;
use crate::error::{CoreError, CoreResult};

pub fn enqueue_outbox(
    conn: &Connection,
    workspace_id: &str,
    input: &OutboxInput,
) -> CoreResult<OutboxJob> {
    input.validate()?;
    if let Some(issue_id) = &input.local_issue_id {
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
    }
    if let Some(existing) = find_by_idempotency(conn, workspace_id, &input.idempotency_key)? {
        return Ok(existing);
    }
    let id = uuid::Uuid::now_v7().to_string();
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO sync_outbox \
         (id, local_issue_id, operation, payload_json, state, attempts, idempotency_key, \
          fingerprint, reconciliation_hint, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, 'pending', 0, ?5, ?6, ?7, ?8, ?8)",
        rusqlite::params![
            id,
            input.local_issue_id,
            input.operation.trim(),
            if input.payload_json.trim().is_empty() {
                "{}"
            } else {
                &input.payload_json
            },
            input.idempotency_key.trim(),
            input.fingerprint.trim(),
            input.reconciliation_hint,
            now,
        ],
    )?;
    get_outbox(conn, workspace_id, &id)
}

pub fn list_outbox(conn: &Connection, workspace_id: &str) -> CoreResult<Vec<OutboxJob>> {
    let mut statement = conn.prepare(
        "SELECT o.id, o.local_issue_id, o.operation, o.payload_json, o.state, o.attempts, \
                o.idempotency_key, o.fingerprint, o.reconciliation_hint, o.last_error, \
                o.created_at, o.updated_at \
         FROM sync_outbox o LEFT JOIN issue i ON i.id = o.local_issue_id \
         WHERE i.workspace_id = ?1 OR (o.local_issue_id IS NULL) \
         ORDER BY o.created_at DESC LIMIT 200",
    )?;
    let rows = statement.query_map([workspace_id], read_outbox)?;
    rows.map(|row| row.map_err(CoreError::from))
        .collect::<CoreResult<Vec<_>>>()
}

pub fn transition_outbox(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    next: OutboxState,
    error: Option<&str>,
) -> CoreResult<OutboxJob> {
    let current = get_outbox(conn, workspace_id, id)?;
    let current_state = OutboxState::parse(&current.state)?;
    if !current_state.can_transition_to(next) {
        return Err(CoreError::conflict(format!(
            "Outbox cannot move from {} to {}.",
            current.state,
            next.as_str()
        )));
    }
    let attempts = if next == OutboxState::Dispatching {
        current.attempts + 1
    } else {
        current.attempts
    };
    conn.execute(
        "UPDATE sync_outbox SET state = ?1, attempts = ?2, last_error = ?3, updated_at = ?4 \
         WHERE id = ?5",
        rusqlite::params![next.as_str(), attempts, error, now_rfc3339(), id],
    )?;
    get_outbox(conn, workspace_id, id)
}

pub fn upsert_link(
    conn: &Connection,
    workspace_id: &str,
    link: &RemoteIssueLink,
) -> CoreResult<RemoteIssueLink> {
    let issue_workspace: Option<String> = conn
        .query_row(
            "SELECT workspace_id FROM issue WHERE id = ?1 AND deleted_at IS NULL",
            [&link.local_issue_id],
            |row| row.get(0),
        )
        .optional()?;
    if issue_workspace.as_deref() != Some(workspace_id) {
        return Err(CoreError::not_found("Issue"));
    }
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO remote_issue_link \
         (id, local_issue_id, provider, account_id, repository_id, remote_id, remote_number, \
          remote_url, remote_updated_at, last_synced_at, field_ownership, state, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13) \
         ON CONFLICT(local_issue_id) DO UPDATE SET provider = excluded.provider, \
          account_id = excluded.account_id, repository_id = excluded.repository_id, \
          remote_id = excluded.remote_id, remote_number = excluded.remote_number, \
          remote_url = excluded.remote_url, remote_updated_at = excluded.remote_updated_at, \
          last_synced_at = excluded.last_synced_at, field_ownership = excluded.field_ownership, \
          state = excluded.state, updated_at = excluded.updated_at",
        rusqlite::params![
            link.id,
            link.local_issue_id,
            link.provider,
            link.account_id,
            link.repository_id,
            link.remote_id,
            link.remote_number,
            link.remote_url,
            link.remote_updated_at,
            link.last_synced_at,
            link.field_ownership,
            link.state,
            now,
        ],
    )?;
    get_link(conn, workspace_id, &link.local_issue_id)?
        .ok_or_else(|| CoreError::internal("Remote link was not persisted."))
}

pub fn get_link(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
) -> CoreResult<Option<RemoteIssueLink>> {
    Ok(conn
        .query_row(
            "SELECT l.id, l.local_issue_id, l.provider, l.account_id, l.repository_id, \
                    l.remote_id, l.remote_number, l.remote_url, l.remote_updated_at, \
                    l.last_synced_at, l.field_ownership, l.state, l.created_at, l.updated_at \
             FROM remote_issue_link l JOIN issue i ON i.id = l.local_issue_id \
             WHERE i.workspace_id = ?1 AND l.local_issue_id = ?2",
            rusqlite::params![workspace_id, issue_id],
            read_link,
        )
        .optional()?)
}

pub fn find_by_remote(
    conn: &Connection,
    workspace_id: &str,
    provider: &str,
    account_id: &str,
    remote_id: &str,
) -> CoreResult<Option<RemoteIssueLink>> {
    Ok(conn
        .query_row(
            "SELECT l.id, l.local_issue_id, l.provider, l.account_id, l.repository_id, \
                    l.remote_id, l.remote_number, l.remote_url, l.remote_updated_at, \
                    l.last_synced_at, l.field_ownership, l.state, l.created_at, l.updated_at \
             FROM remote_issue_link l JOIN issue i ON i.id = l.local_issue_id \
             WHERE i.workspace_id = ?1 AND l.provider = ?2 AND l.account_id = ?3 \
               AND l.remote_id = ?4",
            rusqlite::params![workspace_id, provider, account_id, remote_id],
            read_link,
        )
        .optional()?)
}

pub fn list_conflicts(conn: &Connection, workspace_id: &str) -> CoreResult<Vec<SyncConflict>> {
    let mut statement = conn.prepare(
        "SELECT c.id, c.local_issue_id, c.field, c.local_value, c.remote_value, \
                c.remote_updated_at, c.resolution, c.resolved_at, c.created_at \
         FROM sync_conflict c JOIN issue i ON i.id = c.local_issue_id \
         WHERE i.workspace_id = ?1 AND c.resolved_at IS NULL ORDER BY c.created_at DESC",
    )?;
    let rows = statement.query_map([workspace_id], read_conflict)?;
    rows.map(|row| row.map_err(CoreError::from))
        .collect::<CoreResult<Vec<_>>>()
}

pub fn create_conflict(
    conn: &Connection,
    workspace_id: &str,
    issue_id: &str,
    field: &str,
    local_value: Option<&str>,
    remote_value: Option<&str>,
    remote_updated_at: Option<&str>,
) -> CoreResult<SyncConflict> {
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
    let id = uuid::Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO sync_conflict \
         (id, local_issue_id, field, local_value, remote_value, remote_updated_at, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            id,
            issue_id,
            field,
            local_value,
            remote_value,
            remote_updated_at,
            now_rfc3339(),
        ],
    )?;
    get_conflict(conn, &id)
}

pub fn resolve_conflict(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    resolution: &str,
) -> CoreResult<SyncConflict> {
    let changed = conn.execute(
        "UPDATE sync_conflict SET resolution = ?1, resolved_at = ?2 \
         WHERE id = ?3 AND local_issue_id IN (SELECT id FROM issue WHERE workspace_id = ?4)",
        rusqlite::params![resolution, now_rfc3339(), id, workspace_id],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Sync conflict"));
    }
    get_conflict(conn, id)
}

pub fn upsert_cursor(
    conn: &Connection,
    provider: &str,
    account_id: &str,
    resource: &str,
    scope: &str,
    input: &SyncCursorInput,
) -> CoreResult<SyncCursor> {
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO sync_cursor \
         (id, provider, account_id, resource, scope, page_cursor, watermark, last_success_at, \
          last_attempt_at, next_due_at, last_error, incomplete_reason, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13) \
         ON CONFLICT(provider, account_id, resource, scope) DO UPDATE SET \
          page_cursor = excluded.page_cursor, watermark = excluded.watermark, \
          last_success_at = coalesce(excluded.last_success_at, sync_cursor.last_success_at), \
          last_attempt_at = excluded.last_attempt_at, next_due_at = excluded.next_due_at, \
          last_error = excluded.last_error, incomplete_reason = excluded.incomplete_reason, \
          updated_at = excluded.updated_at",
        rusqlite::params![
            uuid::Uuid::now_v7().to_string(),
            provider,
            account_id,
            resource,
            scope,
            input.page_cursor,
            input.watermark,
            input.last_success_at,
            input.last_attempt_at,
            input.next_due_at,
            input.last_error,
            input.incomplete_reason,
            now,
        ],
    )?;
    get_cursor(conn, provider, account_id, resource, scope)?
        .ok_or_else(|| CoreError::internal("Sync cursor was not persisted."))
}

pub fn get_cursor(
    conn: &Connection,
    provider: &str,
    account_id: &str,
    resource: &str,
    scope: &str,
) -> CoreResult<Option<SyncCursor>> {
    Ok(conn
        .query_row(
            "SELECT id, provider, account_id, resource, scope, page_cursor, watermark, \
                    last_success_at, last_attempt_at, next_due_at, last_error, \
                    incomplete_reason, updated_at \
             FROM sync_cursor WHERE provider = ?1 AND account_id = ?2 AND resource = ?3 AND scope = ?4",
            rusqlite::params![provider, account_id, resource, scope],
            read_cursor,
        )
        .optional()?)
}

fn find_by_idempotency(
    conn: &Connection,
    workspace_id: &str,
    key: &str,
) -> CoreResult<Option<OutboxJob>> {
    Ok(conn
        .query_row(
            "SELECT o.id, o.local_issue_id, o.operation, o.payload_json, o.state, o.attempts, \
                    o.idempotency_key, o.fingerprint, o.reconciliation_hint, o.last_error, \
                    o.created_at, o.updated_at \
             FROM sync_outbox o LEFT JOIN issue i ON i.id = o.local_issue_id \
             WHERE o.idempotency_key = ?1 AND (i.workspace_id = ?2 OR o.local_issue_id IS NULL)",
            rusqlite::params![key, workspace_id],
            read_outbox,
        )
        .optional()?)
}

fn get_outbox(conn: &Connection, workspace_id: &str, id: &str) -> CoreResult<OutboxJob> {
    conn.query_row(
        "SELECT o.id, o.local_issue_id, o.operation, o.payload_json, o.state, o.attempts, \
                o.idempotency_key, o.fingerprint, o.reconciliation_hint, o.last_error, \
                o.created_at, o.updated_at \
         FROM sync_outbox o LEFT JOIN issue i ON i.id = o.local_issue_id \
         WHERE o.id = ?1 AND (i.workspace_id = ?2 OR o.local_issue_id IS NULL)",
        rusqlite::params![id, workspace_id],
        read_outbox,
    )
    .optional()?
    .ok_or_else(|| CoreError::not_found("Sync job"))
}

fn get_conflict(conn: &Connection, id: &str) -> CoreResult<SyncConflict> {
    conn.query_row(
        "SELECT id, local_issue_id, field, local_value, remote_value, remote_updated_at, \
                resolution, resolved_at, created_at FROM sync_conflict WHERE id = ?1",
        [id],
        read_conflict,
    )
    .optional()?
    .ok_or_else(|| CoreError::not_found("Sync conflict"))
}

fn read_outbox(row: &Row<'_>) -> rusqlite::Result<OutboxJob> {
    Ok(OutboxJob {
        id: row.get(0)?,
        local_issue_id: row.get(1)?,
        operation: row.get(2)?,
        payload_json: row.get(3)?,
        state: row.get(4)?,
        attempts: row.get(5)?,
        idempotency_key: row.get(6)?,
        fingerprint: row.get(7)?,
        reconciliation_hint: row.get(8)?,
        last_error: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn read_link(row: &Row<'_>) -> rusqlite::Result<RemoteIssueLink> {
    Ok(RemoteIssueLink {
        id: row.get(0)?,
        local_issue_id: row.get(1)?,
        provider: row.get(2)?,
        account_id: row.get(3)?,
        repository_id: row.get(4)?,
        remote_id: row.get(5)?,
        remote_number: row.get(6)?,
        remote_url: row.get(7)?,
        remote_updated_at: row.get(8)?,
        last_synced_at: row.get(9)?,
        field_ownership: row.get(10)?,
        state: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

fn read_conflict(row: &Row<'_>) -> rusqlite::Result<SyncConflict> {
    Ok(SyncConflict {
        id: row.get(0)?,
        local_issue_id: row.get(1)?,
        field: row.get(2)?,
        local_value: row.get(3)?,
        remote_value: row.get(4)?,
        remote_updated_at: row.get(5)?,
        resolution: row.get(6)?,
        resolved_at: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn read_cursor(row: &Row<'_>) -> rusqlite::Result<SyncCursor> {
    Ok(SyncCursor {
        id: row.get(0)?,
        provider: row.get(1)?,
        account_id: row.get(2)?,
        resource: row.get(3)?,
        scope: row.get(4)?,
        page_cursor: row.get(5)?,
        watermark: row.get(6)?,
        last_success_at: row.get(7)?,
        last_attempt_at: row.get(8)?,
        next_due_at: row.get(9)?,
        last_error: row.get(10)?,
        incomplete_reason: row.get(11)?,
        updated_at: row.get(12)?,
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
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "Sync"))
            .expect("workspace");
        let issue = db
            .with_conn(|conn| {
                crate::db::repositories::issues::create(
                    conn,
                    &workspace.id,
                    &CreateIssueInput {
                        title: "Sync me".into(),
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
    fn outbox_idempotency_and_ambiguous_state_machine() {
        let (_dir, db, workspace_id, issue_id) = fixture();
        let input = OutboxInput {
            local_issue_id: Some(issue_id.clone()),
            operation: "issue.create".into(),
            payload_json: "{\"title\":\"Sync me\"}".into(),
            idempotency_key: "issue-create-1".into(),
            fingerprint: "sha256:abc".into(),
            reconciliation_hint: Some("search by title".into()),
        };
        let first = db
            .with_conn(|conn| enqueue_outbox(conn, &workspace_id, &input))
            .expect("enqueue");
        let second = db
            .with_conn(|conn| enqueue_outbox(conn, &workspace_id, &input))
            .expect("idempotent enqueue");
        assert_eq!(first.id, second.id);

        let dispatching = db
            .with_conn(|conn| {
                transition_outbox(
                    conn,
                    &workspace_id,
                    &first.id,
                    OutboxState::Dispatching,
                    None,
                )
            })
            .expect("dispatch");
        assert_eq!(dispatching.attempts, 1);
        let unknown = db
            .with_conn(|conn| {
                transition_outbox(
                    conn,
                    &workspace_id,
                    &first.id,
                    OutboxState::OutcomeUnknown,
                    Some("timeout"),
                )
            })
            .expect("unknown");
        assert_eq!(unknown.state, "outcome_unknown");
        assert!(db
            .with_conn(|conn| {
                transition_outbox(conn, &workspace_id, &first.id, OutboxState::Succeeded, None)
            })
            .is_err());
        db.with_conn(|conn| {
            transition_outbox(
                conn,
                &workspace_id,
                &first.id,
                OutboxState::Reconciling,
                None,
            )
        })
        .expect("reconcile");
        let retry = db
            .with_conn(|conn| {
                transition_outbox(conn, &workspace_id, &first.id, OutboxState::RetrySafe, None)
            })
            .expect("retry safe");
        assert_eq!(retry.state, "retry_safe");
        let queued = db
            .with_conn(|conn| {
                transition_outbox(conn, &workspace_id, &first.id, OutboxState::Pending, None)
            })
            .expect("requeue");
        assert_eq!(queued.state, "pending");
    }

    #[test]
    fn links_cursors_and_conflicts_persist() {
        let (_dir, db, workspace_id, issue_id) = fixture();
        let link = db
            .with_conn(|conn| {
                upsert_link(
                    conn,
                    &workspace_id,
                    &RemoteIssueLink {
                        id: "link-1".into(),
                        local_issue_id: issue_id.clone(),
                        provider: "github".into(),
                        account_id: "acct".into(),
                        repository_id: Some("repo".into()),
                        remote_id: "R1".into(),
                        remote_number: 42,
                        remote_url: "https://github.com/o/r/issues/42".into(),
                        remote_updated_at: None,
                        last_synced_at: Some("2026-11-15T00:00:00Z".into()),
                        field_ownership: "{}".into(),
                        state: "linked".into(),
                        created_at: String::new(),
                        updated_at: String::new(),
                    },
                )
            })
            .expect("link");
        assert_eq!(link.remote_number, 42);
        assert_eq!(
            db.with_conn(|conn| get_link(conn, &workspace_id, &issue_id))
                .expect("get")
                .expect("some")
                .remote_number,
            42
        );

        let cursor = db
            .with_conn(|conn| {
                upsert_cursor(
                    conn,
                    "github",
                    "acct",
                    "issues",
                    "repo",
                    &SyncCursorInput {
                        watermark: Some("2026-11-15T00:00:00Z".into()),
                        last_success_at: Some("2026-11-15T00:00:00Z".into()),
                        ..Default::default()
                    },
                )
            })
            .expect("cursor");
        assert_eq!(cursor.watermark.as_deref(), Some("2026-11-15T00:00:00Z"));

        let conflict = db
            .with_conn(|conn| {
                create_conflict(
                    conn,
                    &workspace_id,
                    &issue_id,
                    "title",
                    Some("Local"),
                    Some("Remote"),
                    None,
                )
            })
            .expect("conflict");
        let resolved = db
            .with_conn(|conn| resolve_conflict(conn, &workspace_id, &conflict.id, "remote"))
            .expect("resolve");
        assert_eq!(resolved.resolution.as_deref(), Some("remote"));
        assert!(db
            .with_conn(|conn| list_conflicts(conn, &workspace_id))
            .expect("list")
            .is_empty());
    }
}
