use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};

pub const BACKGROUND_PERCENT: f64 = 0.10;
pub const BACKGROUND_RESERVE_PERCENT: f64 = 0.20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiFamily {
    GithubRest,
    GithubGraphql,
    GithubSearch,
    GithubCodeSearch,
    Unknown,
}

impl ApiFamily {
    pub fn parse(raw: &str) -> Self {
        match raw {
            "github_rest" => Self::GithubRest,
            "github_graphql" => Self::GithubGraphql,
            "github_search" => Self::GithubSearch,
            "github_code_search" => Self::GithubCodeSearch,
            _ => Self::Unknown,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GithubRest => "github_rest",
            Self::GithubGraphql => "github_graphql",
            Self::GithubSearch => "github_search",
            Self::GithubCodeSearch => "github_code_search",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetObservation {
    pub id: String,
    pub provider_account_id: Option<String>,
    pub api_family: String,
    pub resource_name: String,
    pub limit_value: Option<i64>,
    pub remaining_value: Option<i64>,
    pub used_value: Option<i64>,
    pub reset_at_utc: Option<String>,
    pub observed_at_utc: String,
    pub http_status: Option<i64>,
    pub source: String,
    pub is_estimate: bool,
    pub secondary_pause_until: Option<String>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetSummary {
    pub api_family: String,
    pub resource_name: String,
    pub limit_value: Option<i64>,
    pub remaining_value: Option<i64>,
    pub used_value: Option<i64>,
    pub reset_at_utc: Option<String>,
    pub observed_at_utc: String,
    pub http_status: Option<i64>,
    pub source: String,
    pub is_estimate: bool,
    pub secondary_pause_until: Option<String>,
    pub background_allowance: i64,
    pub background_reserve: i64,
    pub observed_requests: i64,
}

impl BudgetSummary {
    pub fn from_observation(observation: &BudgetObservation, observed_requests: i64) -> Self {
        let allowance = background_allowance(observation.limit_value, observation.remaining_value);
        let reserve = observation
            .limit_value
            .map(|limit| (limit as f64 * BACKGROUND_RESERVE_PERCENT).round() as i64)
            .unwrap_or(0);
        Self {
            api_family: observation.api_family.clone(),
            resource_name: observation.resource_name.clone(),
            limit_value: observation.limit_value,
            remaining_value: observation.remaining_value,
            used_value: observation.used_value,
            reset_at_utc: observation.reset_at_utc.clone(),
            observed_at_utc: observation.observed_at_utc.clone(),
            http_status: observation.http_status,
            source: observation.source.clone(),
            is_estimate: observation.is_estimate,
            secondary_pause_until: observation.secondary_pause_until.clone(),
            background_allowance: allowance,
            background_reserve: reserve,
            observed_requests,
        }
    }
}

/// `min(10% of observed limit, max(0, remaining - 20% of limit))`.
/// Unknown limit/remaining yields zero background allowance.
pub fn background_allowance(limit: Option<i64>, remaining: Option<i64>) -> i64 {
    let (Some(limit), Some(remaining)) = (limit, remaining) else {
        return 0;
    };
    if limit <= 0 {
        return 0;
    }
    let ten_percent = (limit as f64 * BACKGROUND_PERCENT).floor() as i64;
    let available = remaining - (limit as f64 * BACKGROUND_RESERVE_PERCENT).round() as i64;
    ten_percent.min(available.max(0))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestLog {
    pub id: String,
    pub provider_account_id: Option<String>,
    pub operation_name: String,
    pub operation_code: String,
    pub api_family: String,
    pub resource_name: String,
    pub request_class: String,
    pub fingerprint: String,
    pub estimated_cost: f64,
    pub actual_cost: f64,
    pub http_status: Option<i64>,
    pub error_category: Option<String>,
    pub latency_ms: Option<i64>,
    pub received_at_utc: String,
    pub retry_attempt: i64,
    pub result_state: String,
    pub rate_headers_json: String,
    pub payload_bytes: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueJob {
    pub id: String,
    pub operation_name: String,
    pub operation_code: String,
    pub api_family: String,
    pub priority: i64,
    pub request_class: String,
    pub state: String,
    pub attempts: i64,
    pub estimated_cost: f64,
    pub fingerprint: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiControl {
    pub paused: bool,
    pub reason: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiMonitorState {
    pub connected: bool,
    pub account_login: Option<String>,
    pub budgets: Vec<BudgetSummary>,
    pub requests: Vec<RequestLog>,
    pub queue: Vec<QueueJob>,
    pub control: ApiControl,
    pub last_observation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiObservationInput {
    pub api_family: String,
    pub resource_name: String,
    pub limit_value: Option<i64>,
    pub remaining_value: Option<i64>,
    pub used_value: Option<i64>,
    pub reset_at_utc: Option<String>,
    pub http_status: Option<i64>,
    pub source: String,
    pub is_estimate: bool,
    pub secondary_pause_until: Option<String>,
    pub notes: String,
}

impl ApiObservationInput {
    pub fn validate(&self) -> CoreResult<()> {
        if self.resource_name.trim().is_empty() {
            return Err(CoreError::validation(
                "resource",
                "Resource name is required.",
            ));
        }
        if !matches!(
            self.source.as_str(),
            "response_headers" | "graphql_rate_limit" | "manual_probe"
        ) {
            return Err(CoreError::validation(
                "source",
                "Unknown observation source.",
            ));
        }
        if let (Some(limit), Some(remaining)) = (self.limit_value, self.remaining_value) {
            if remaining < 0 || limit < 0 || remaining > limit {
                return Err(CoreError::validation(
                    "remaining",
                    "Remaining value must be between 0 and the limit.",
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_allowance_uses_stricter_bound() {
        assert_eq!(background_allowance(Some(5000), Some(5000)), 500);
        assert_eq!(background_allowance(Some(5000), Some(1200)), 200);
        assert_eq!(background_allowance(Some(5000), Some(900)), 0);
        assert_eq!(background_allowance(None, Some(900)), 0);
        assert_eq!(background_allowance(Some(5000), None), 0);
        assert_eq!(background_allowance(Some(0), Some(0)), 0);
    }

    #[test]
    fn summary_marks_estimate_and_reserve() {
        let observation = BudgetObservation {
            id: "o".into(),
            provider_account_id: None,
            api_family: "github_graphql".into(),
            resource_name: "graphql".into(),
            limit_value: Some(5000),
            remaining_value: Some(4000),
            used_value: Some(1000),
            reset_at_utc: None,
            observed_at_utc: "2026-11-15T00:00:00Z".into(),
            http_status: Some(200),
            source: "response_headers".into(),
            is_estimate: false,
            secondary_pause_until: None,
            notes: String::new(),
        };
        let summary = BudgetSummary::from_observation(&observation, 12);
        assert_eq!(summary.background_allowance, 500);
        assert_eq!(summary.background_reserve, 1000);
        assert_eq!(summary.observed_requests, 12);
        assert!(!summary.is_estimate);
    }

    #[test]
    fn rejects_invalid_observation() {
        let input = ApiObservationInput {
            api_family: "github_rest".into(),
            resource_name: "core".into(),
            limit_value: Some(10),
            remaining_value: Some(11),
            used_value: None,
            reset_at_utc: None,
            http_status: None,
            source: "response_headers".into(),
            is_estimate: false,
            secondary_pause_until: None,
            notes: String::new(),
        };
        assert!(input.validate().is_err());
    }
}
