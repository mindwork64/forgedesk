use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::domain::time::now_rfc3339;
use crate::error::{CoreError, CoreResult};
use crate::github_client::RateSnapshot;

static DATABASE_PATH: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
static REQUEST_GATE: Mutex<()> = Mutex::new(());
static LAST_MUTATION: Mutex<Option<Instant>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestClass {
    InteractiveRead,
    InteractiveWrite,
    ForegroundRefresh,
    BackgroundSync,
    ManualFullSync,
    Reconciliation,
}

impl RequestClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InteractiveRead => "interactive_read",
            Self::InteractiveWrite => "interactive_write",
            Self::ForegroundRefresh => "foreground_refresh",
            Self::BackgroundSync => "background_sync",
            Self::ManualFullSync => "manual_full_sync",
            Self::Reconciliation => "reconciliation",
        }
    }

    fn is_background(self) -> bool {
        matches!(
            self,
            Self::BackgroundSync | Self::ManualFullSync | Self::Reconciliation
        )
    }
}

#[derive(Debug, Serialize)]
struct RateHeaders<'a> {
    limit: Option<i64>,
    remaining: Option<i64>,
    used: Option<i64>,
    reset: Option<i64>,
    resource: &'a Option<String>,
    retry_after_seconds: Option<u64>,
}

pub fn configure_runtime_database(path: impl Into<PathBuf>) -> CoreResult<()> {
    let lock = DATABASE_PATH.get_or_init(|| Mutex::new(None));
    let mut current = lock
        .lock()
        .map_err(|_| CoreError::internal("GitHub dispatcher configuration is unavailable."))?;
    *current = Some(path.into());
    Ok(())
}

pub(crate) fn request_gate() -> MutexGuard<'static, ()> {
    REQUEST_GATE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(crate) fn pace_mutation() {
    let mut last = LAST_MUTATION
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(previous) = *last {
        let remaining = Duration::from_secs(1).saturating_sub(previous.elapsed());
        if !remaining.is_zero() {
            std::thread::sleep(remaining);
        }
    }
    *last = Some(Instant::now());
}

fn configured_path() -> CoreResult<Option<PathBuf>> {
    let Some(lock) = DATABASE_PATH.get() else {
        return Ok(None);
    };
    let current = lock
        .lock()
        .map_err(|_| CoreError::internal("GitHub dispatcher configuration is unavailable."))?;
    Ok(current.clone())
}

fn open_runtime_database(path: &PathBuf) -> CoreResult<Connection> {
    let connection = Connection::open(path)?;
    connection.busy_timeout(Duration::from_secs(3))?;
    Ok(connection)
}

fn workspace_id(connection: &Connection) -> CoreResult<Option<String>> {
    Ok(connection
        .query_row(
            "SELECT id FROM workspace ORDER BY created_at LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?)
}

fn inferred_resource(path: &str) -> &'static str {
    if path.starts_with("/search/") {
        "search"
    } else {
        "core"
    }
}

pub(crate) fn authorize(class: RequestClass, path: &str) -> CoreResult<()> {
    let Some(database_path) = configured_path()? else {
        return Ok(());
    };
    let connection = open_runtime_database(&database_path)?;
    let Some(workspace_id) = workspace_id(&connection)? else {
        return Ok(());
    };

    let paused: bool = connection
        .query_row(
            "SELECT paused FROM api_control WHERE workspace_id = ?1",
            [&workspace_id],
            |row| row.get::<_, i64>(0).map(|value| value != 0),
        )
        .optional()?
        .unwrap_or(false);
    if paused {
        return Err(CoreError::rate_limit_secondary(
            "GitHub requests are paused in API Monitor. Resume requests to continue.",
        ));
    }

    let resource = inferred_resource(path);
    let observation: Option<(Option<i64>, Option<i64>, Option<String>, String, Option<String>)> =
        connection.query_row(
            "SELECT remaining_value, limit_value, reset_at_utc, observed_at_utc, secondary_pause_until
             FROM api_budget_observation
             WHERE workspace_id = ?1 AND api_family = 'github_rest' AND resource_name = ?2
             ORDER BY observed_at_utc DESC, id DESC LIMIT 1",
            rusqlite::params![workspace_id, resource],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        ).optional()?;

    let Some((remaining, _limit, reset_at, observed_at, secondary_pause_until)) = observation
    else {
        return Ok(());
    };

    let now = Utc::now();
    if let Some(until) = secondary_pause_until.as_deref() {
        if DateTime::parse_from_rfc3339(until)
            .map(|value| value.with_timezone(&Utc) > now)
            .unwrap_or(false)
        {
            return Err(CoreError::rate_limit_secondary(format!(
                "GitHub secondary rate limit is active until {until}."
            )));
        }
    }

    let reset_in_future = reset_at
        .as_deref()
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&Utc) > now)
        .unwrap_or(false);
    if reset_in_future {
        let available = remaining.unwrap_or(0);
        let consumed_since_observation: i64 = connection.query_row(
            "SELECT count(*) FROM api_request_log
             WHERE workspace_id = ?1 AND api_family = 'github_rest' AND resource_name = ?2
               AND received_at_utc > ?3",
            rusqlite::params![workspace_id, resource, observed_at],
            |row| row.get(0),
        )?;
        if available.saturating_sub(consumed_since_observation) <= 0 {
            let label = if class.is_background() {
                "Background"
            } else {
                "Interactive"
            };
            return Err(CoreError::rate_limit_primary(format!(
                "{label} GitHub requests are paused until the REST rate-limit reset."
            )));
        }
    }

    Ok(())
}

pub(crate) fn record_response(
    class: RequestClass,
    method: &str,
    path: &str,
    body: Option<&serde_json::Value>,
    status: u16,
    rate: &RateSnapshot,
    retry_after_seconds: Option<u64>,
    elapsed: Duration,
    response_bytes: usize,
    response_body: &str,
) -> CoreResult<()> {
    let Some(database_path) = configured_path()? else {
        return Ok(());
    };
    let mut connection = open_runtime_database(&database_path)?;
    let Some(workspace_id) = workspace_id(&connection)? else {
        return Ok(());
    };

    let resource = rate
        .resource
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| inferred_resource(path));
    let body_lower = response_body.to_ascii_lowercase();
    let secondary_limited = status == 429
        || (status == 403
            && (body_lower.contains("secondary rate limit")
                || body_lower.contains("abuse detection")));
    let reset_at = rate
        .reset
        .and_then(|value| DateTime::<Utc>::from_timestamp(value, 0))
        .map(|value| value.to_rfc3339());
    let secondary_pause_until = if secondary_limited {
        let seconds = retry_after_seconds.unwrap_or(60).clamp(1, 3600);
        Some((Utc::now() + chrono::Duration::seconds(seconds as i64)).to_rfc3339())
    } else {
        None
    };
    let result_state = if secondary_limited {
        "secondary_limited"
    } else if status == 403 && rate.remaining == Some(0) {
        "rate_limited"
    } else if (200..300).contains(&status) {
        "success"
    } else {
        "failed"
    };
    let error_category = match result_state {
        "secondary_limited" => Some("rate_limit_secondary"),
        "rate_limited" => Some("rate_limit_primary"),
        "failed" => Some("provider"),
        _ => None,
    };
    let operation_name = format!("{} {}", method, path.split('?').next().unwrap_or(path));
    let operation_code = format!("{}_{}", method.to_ascii_lowercase(), resource);
    let body_text = body.map(serde_json::Value::to_string).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(method.as_bytes());
    hasher.update(b"\n");
    hasher.update(path.as_bytes());
    hasher.update(b"\n");
    hasher.update(body_text.as_bytes());
    let fingerprint = format!("{:x}", hasher.finalize());
    let now = now_rfc3339();
    let rate_headers = serde_json::to_string(&RateHeaders {
        limit: rate.limit,
        remaining: rate.remaining,
        used: rate.used,
        reset: rate.reset,
        resource: &rate.resource,
        retry_after_seconds,
    })
    .unwrap_or_else(|_| "{}".into());

    let tx = connection.transaction()?;
    if rate.limit.is_some() || rate.remaining.is_some() || secondary_pause_until.is_some() {
        tx.execute(
            "INSERT INTO api_budget_observation
             (id, workspace_id, provider_account_id, api_family, resource_name, limit_value,
              remaining_value, used_value, reset_at_utc, observed_at_utc, http_status, source,
              is_estimate, secondary_pause_until, notes)
             VALUES (?1, ?2, NULL, 'github_rest', ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                     'response_headers', 0, ?10, ?11)",
            rusqlite::params![
                uuid::Uuid::now_v7().to_string(),
                workspace_id,
                resource,
                rate.limit,
                rate.remaining,
                rate.used,
                reset_at,
                now,
                status as i64,
                secondary_pause_until,
                format!("{} {}", method, path.split('?').next().unwrap_or(path)),
            ],
        )?;
    }
    tx.execute(
        "INSERT INTO api_request_log
         (id, workspace_id, provider_account_id, operation_name, operation_code, api_family,
          resource_name, request_class, fingerprint, estimated_cost, actual_cost, http_status,
          error_category, latency_ms, received_at_utc, retry_attempt, result_state,
          rate_headers_json, payload_bytes)
         VALUES (?1, ?2, NULL, ?3, ?4, 'github_rest', ?5, ?6, ?7, 1.0, ?8, ?9,
                 ?10, ?11, ?12, 0, ?13, ?14, ?15)",
        rusqlite::params![
            uuid::Uuid::now_v7().to_string(),
            workspace_id,
            operation_name,
            operation_code,
            resource,
            class.as_str(),
            fingerprint,
            if (200..300).contains(&status) {
                1.0
            } else {
                0.0
            },
            status as i64,
            error_category,
            elapsed.as_millis().min(i64::MAX as u128) as i64,
            now,
            result_state,
            rate_headers,
            response_bytes as i64,
        ],
    )?;
    tx.commit()?;
    Ok(())
}

pub(crate) fn record_transport_failure(
    class: RequestClass,
    method: &str,
    path: &str,
    body: Option<&serde_json::Value>,
    elapsed: Duration,
    error: &str,
) -> CoreResult<()> {
    let Some(database_path) = configured_path()? else { return Ok(()); };
    let connection = open_runtime_database(&database_path)?;
    let Some(workspace_id) = workspace_id(&connection)? else { return Ok(()); };
    let resource = inferred_resource(path);
    let body_text = body.map(serde_json::Value::to_string).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(method.as_bytes()); hasher.update(b"\n");
    hasher.update(path.as_bytes()); hasher.update(b"\n");
    hasher.update(body_text.as_bytes());
    let fingerprint = format!("{:x}", hasher.finalize());
    connection.execute(
        "INSERT INTO api_request_log
         (id, workspace_id, provider_account_id, operation_name, operation_code, api_family,
          resource_name, request_class, fingerprint, estimated_cost, actual_cost, http_status,
          error_category, latency_ms, received_at_utc, retry_attempt, result_state,
          rate_headers_json, payload_bytes)
         VALUES (?1, ?2, NULL, ?3, ?4, 'github_rest', ?5, ?6, ?7, 1.0, 0.0, NULL,
                 'network', ?8, ?9, 0, 'failed', '{}', NULL)",
        rusqlite::params![
            uuid::Uuid::now_v7().to_string(),
            workspace_id,
            format!("{} {}", method, path.split('?').next().unwrap_or(path)),
            format!("{}_{}", method.to_ascii_lowercase(), resource),
            resource,
            class.as_str(),
            fingerprint,
            elapsed.as_millis().min(i64::MAX as u128) as i64,
            now_rfc3339(),
        ],
    )?;
    let _ = error;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persisted_request_classes_match_schema() {
        for class in [
            RequestClass::InteractiveRead,
            RequestClass::InteractiveWrite,
            RequestClass::ForegroundRefresh,
            RequestClass::BackgroundSync,
            RequestClass::ManualFullSync,
            RequestClass::Reconciliation,
        ] {
            assert!(matches!(
                class.as_str(),
                "interactive_read"
                    | "interactive_write"
                    | "foreground_refresh"
                    | "background_sync"
                    | "manual_full_sync"
                    | "reconciliation"
            ));
        }
    }
}