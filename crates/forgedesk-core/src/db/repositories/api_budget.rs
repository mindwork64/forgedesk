use rusqlite::{Connection, OptionalExtension, Row};

use crate::domain::api_budget::{
    ApiControl, ApiFamily, ApiObservationInput, BudgetObservation, BudgetSummary, QueueJob,
    RequestLog,
};
use crate::domain::time::now_rfc3339;
use crate::error::{CoreError, CoreResult};

pub fn record_observation(
    conn: &Connection,
    workspace_id: &str,
    input: &ApiObservationInput,
) -> CoreResult<BudgetObservation> {
    input.validate()?;
    let id = uuid::Uuid::now_v7().to_string();
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO api_budget_observation \
         (id, workspace_id, provider_account_id, api_family, resource_name, limit_value, \
          remaining_value, used_value, reset_at_utc, observed_at_utc, http_status, source, \
          is_estimate, secondary_pause_until, notes) \
         VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        rusqlite::params![
            id,
            workspace_id,
            ApiFamily::parse(&input.api_family).as_str(),
            input.resource_name.trim(),
            input.limit_value,
            input.remaining_value,
            input.used_value,
            input.reset_at_utc,
            now,
            input.http_status,
            input.source,
            i64::from(input.is_estimate),
            input.secondary_pause_until,
            input.notes,
        ],
    )?;
    get_observation(conn, workspace_id, &id)
}

pub fn list_budgets(conn: &Connection, workspace_id: &str) -> CoreResult<Vec<BudgetSummary>> {
    let mut statement = conn.prepare(
        "SELECT o.id, o.api_family, o.resource_name, o.limit_value, o.remaining_value, \
                o.used_value, o.reset_at_utc, o.observed_at_utc, o.http_status, o.source, \
                o.is_estimate, o.secondary_pause_until, o.notes, \
                (SELECT count(*) FROM api_request_log l WHERE l.workspace_id = o.workspace_id \
                   AND l.api_family = o.api_family AND l.resource_name = o.resource_name) \
         FROM api_budget_observation o \
         WHERE o.workspace_id = ?1 AND o.id IN ( \
             SELECT id FROM api_budget_observation o2 \
             WHERE o2.workspace_id = o.workspace_id AND o2.api_family = o.api_family \
               AND o2.resource_name = o.resource_name \
             ORDER BY o2.observed_at_utc DESC, o2.id DESC LIMIT 1) \
         ORDER BY o.api_family, o.resource_name",
    )?;
    let rows = statement.query_map([workspace_id], |row| {
        let observation = BudgetObservation {
            id: row.get(0)?,
            provider_account_id: None,
            api_family: row.get(1)?,
            resource_name: row.get(2)?,
            limit_value: row.get(3)?,
            remaining_value: row.get(4)?,
            used_value: row.get(5)?,
            reset_at_utc: row.get(6)?,
            observed_at_utc: row.get(7)?,
            http_status: row.get(8)?,
            source: row.get(9)?,
            is_estimate: row.get::<_, i64>(10)? != 0,
            secondary_pause_until: row.get(11)?,
            notes: row.get(12)?,
        };
        Ok(BudgetSummary::from_observation(&observation, row.get(13)?))
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn list_requests(conn: &Connection, workspace_id: &str) -> CoreResult<Vec<RequestLog>> {
    let mut statement = conn.prepare(
        "SELECT id, provider_account_id, operation_name, operation_code, api_family, \
                resource_name, request_class, fingerprint, estimated_cost, actual_cost, \
                http_status, error_category, latency_ms, received_at_utc, retry_attempt, \
                result_state, rate_headers_json, payload_bytes \
         FROM api_request_log WHERE workspace_id = ?1 ORDER BY received_at_utc DESC LIMIT 100",
    )?;
    let rows = statement.query_map([workspace_id], read_request)?;
    rows.map(|row| row.map_err(CoreError::from))
        .collect::<CoreResult<Vec<_>>>()
}

pub fn list_queue(conn: &Connection, workspace_id: &str) -> CoreResult<Vec<QueueJob>> {
    let mut statement = conn.prepare(
        "SELECT id, operation_name, operation_code, api_family, priority, request_class, state, \
                attempts, estimated_cost, fingerprint, created_at, updated_at, last_error \
         FROM api_queue_job WHERE workspace_id = ?1 \
         ORDER BY CASE state WHEN 'running' THEN 0 WHEN 'queued' THEN 1 ELSE 2 END, priority, created_at \
         LIMIT 100",
    )?;
    let rows = statement.query_map([workspace_id], |row| {
        Ok(QueueJob {
            id: row.get(0)?,
            operation_name: row.get(1)?,
            operation_code: row.get(2)?,
            api_family: row.get(3)?,
            priority: row.get(4)?,
            request_class: row.get(5)?,
            state: row.get(6)?,
            attempts: row.get(7)?,
            estimated_cost: row.get(8)?,
            fingerprint: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            last_error: row.get(12)?,
        })
    })?;
    rows.map(|row| row.map_err(CoreError::from))
        .collect::<CoreResult<Vec<_>>>()
}

pub fn control(conn: &Connection, workspace_id: &str) -> CoreResult<ApiControl> {
    Ok(conn
        .query_row(
            "SELECT paused, reason, updated_at FROM api_control WHERE workspace_id = ?1",
            [workspace_id],
            |row| {
                Ok(ApiControl {
                    paused: row.get::<_, i64>(0)? != 0,
                    reason: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            },
        )
        .optional()?
        .unwrap_or_else(|| ApiControl {
            paused: false,
            reason: String::new(),
            updated_at: now_rfc3339(),
        }))
}

pub fn set_paused(
    conn: &Connection,
    workspace_id: &str,
    paused: bool,
    reason: &str,
) -> CoreResult<ApiControl> {
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO api_control (workspace_id, paused, reason, updated_at) \
         VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT(workspace_id) DO UPDATE SET paused = excluded.paused, \
         reason = excluded.reason, updated_at = excluded.updated_at",
        rusqlite::params![workspace_id, i64::from(paused), reason, now],
    )?;
    control(conn, workspace_id)
}

pub fn account_login(conn: &Connection, workspace_id: &str) -> CoreResult<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT account_login FROM provider_account WHERE workspace_id = ?1 AND provider = 'github' \
             AND status = 'connected' ORDER BY connected_at DESC LIMIT 1",
            [workspace_id],
            |row| row.get(0),
        )
        .optional()?)
}

pub fn last_observation(conn: &Connection, workspace_id: &str) -> CoreResult<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT observed_at_utc FROM api_budget_observation WHERE workspace_id = ?1 \
             ORDER BY observed_at_utc DESC LIMIT 1",
            [workspace_id],
            |row| row.get(0),
        )
        .optional()?)
}

fn get_observation(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
) -> CoreResult<BudgetObservation> {
    conn.query_row(
        "SELECT id, api_family, resource_name, limit_value, remaining_value, used_value, \
                reset_at_utc, observed_at_utc, http_status, source, is_estimate, \
                secondary_pause_until, notes \
         FROM api_budget_observation WHERE workspace_id = ?1 AND id = ?2",
        rusqlite::params![workspace_id, id],
        |row| {
            Ok(BudgetObservation {
                id: row.get(0)?,
                provider_account_id: None,
                api_family: row.get(1)?,
                resource_name: row.get(2)?,
                limit_value: row.get(3)?,
                remaining_value: row.get(4)?,
                used_value: row.get(5)?,
                reset_at_utc: row.get(6)?,
                observed_at_utc: row.get(7)?,
                http_status: row.get(8)?,
                source: row.get(9)?,
                is_estimate: row.get::<_, i64>(10)? != 0,
                secondary_pause_until: row.get(11)?,
                notes: row.get(12)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| CoreError::not_found("Budget observation"))
}

fn read_request(row: &Row<'_>) -> rusqlite::Result<RequestLog> {
    Ok(RequestLog {
        id: row.get(0)?,
        provider_account_id: row.get(1)?,
        operation_name: row.get(2)?,
        operation_code: row.get(3)?,
        api_family: row.get(4)?,
        resource_name: row.get(5)?,
        request_class: row.get(6)?,
        fingerprint: row.get(7)?,
        estimated_cost: row.get(8)?,
        actual_cost: row.get(9)?,
        http_status: row.get(10)?,
        error_category: row.get(11)?,
        latency_ms: row.get(12)?,
        received_at_utc: row.get(13)?,
        retry_attempt: row.get(14)?,
        result_state: row.get(15)?,
        rate_headers_json: row.get(16)?,
        payload_bytes: row.get(17)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::domain::api_budget::ApiObservationInput;

    fn fixture() -> (tempfile::TempDir, Database, String) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("db");
        let workspace = db
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "Budget"))
            .expect("workspace");
        (dir, db, workspace.id)
    }

    fn observation() -> ApiObservationInput {
        ApiObservationInput {
            api_family: "github_graphql".into(),
            resource_name: "graphql".into(),
            limit_value: Some(5000),
            remaining_value: Some(4200),
            used_value: Some(800),
            reset_at_utc: Some("2026-11-16T00:00:00Z".into()),
            http_status: Some(200),
            source: "response_headers".into(),
            is_estimate: false,
            secondary_pause_until: None,
            notes: String::new(),
        }
    }

    #[test]
    fn records_latest_budget_and_counts_observed_requests() {
        let (_dir, db, workspace_id) = fixture();
        db.with_conn(|conn| record_observation(conn, &workspace_id, &observation()))
            .expect("observation");
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO api_request_log \
                 (id, workspace_id, provider_account_id, operation_name, operation_code, api_family, \
                  resource_name, request_class, fingerprint, estimated_cost, actual_cost, http_status, \
                  error_category, latency_ms, received_at_utc, retry_attempt, result_state, \
                  rate_headers_json, payload_bytes) \
                 VALUES ('r1', ?1, NULL, 'IssueList', 'issues.list', 'github_graphql', 'graphql', \
                  'interactive_read', 'sha256:x', 1, 1, 200, NULL, 100, ?2, 0, 'success', '{}', 100)",
                rusqlite::params![workspace_id, "2026-11-15T00:00:00Z"],
            )?;
            Ok(())
        })
        .expect("request");
        let budgets = db
            .with_conn(|conn| list_budgets(conn, &workspace_id))
            .expect("budgets");
        assert_eq!(budgets.len(), 1);
        assert_eq!(budgets[0].remaining_value, Some(4200));
        assert_eq!(budgets[0].observed_requests, 1);
        assert_eq!(budgets[0].background_allowance, 500);
        assert_eq!(budgets[0].background_reserve, 1000);
    }

    #[test]
    fn pause_state_persists() {
        let (_dir, db, workspace_id) = fixture();
        let initial = db
            .with_conn(|conn| control(conn, &workspace_id))
            .expect("control");
        assert!(!initial.paused);
        let paused = db
            .with_conn(|conn| set_paused(conn, &workspace_id, true, "test"))
            .expect("pause");
        assert!(paused.paused);
        assert_eq!(paused.reason, "test");
    }
}
