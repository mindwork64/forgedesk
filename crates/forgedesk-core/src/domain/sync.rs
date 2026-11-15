use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxState {
    Pending,
    Dispatching,
    Succeeded,
    FailedDefinite,
    OutcomeUnknown,
    Reconciling,
    RetrySafe,
    NeedsUserAction,
    Cancelled,
}

impl OutboxState {
    pub fn parse(raw: &str) -> CoreResult<Self> {
        match raw {
            "pending" => Ok(Self::Pending),
            "dispatching" => Ok(Self::Dispatching),
            "succeeded" => Ok(Self::Succeeded),
            "failed_definite" => Ok(Self::FailedDefinite),
            "outcome_unknown" => Ok(Self::OutcomeUnknown),
            "reconciling" => Ok(Self::Reconciling),
            "retry_safe" => Ok(Self::RetrySafe),
            "needs_user_action" => Ok(Self::NeedsUserAction),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(CoreError::internal(format!("Unknown outbox state: {raw}"))),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Dispatching => "dispatching",
            Self::Succeeded => "succeeded",
            Self::FailedDefinite => "failed_definite",
            Self::OutcomeUnknown => "outcome_unknown",
            Self::Reconciling => "reconciling",
            Self::RetrySafe => "retry_safe",
            Self::NeedsUserAction => "needs_user_action",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Pending, Self::Dispatching | Self::Cancelled)
                | (
                    Self::Dispatching,
                    Self::Succeeded | Self::FailedDefinite | Self::OutcomeUnknown | Self::Cancelled
                )
                | (
                    Self::FailedDefinite,
                    Self::RetrySafe | Self::NeedsUserAction
                )
                | (Self::OutcomeUnknown, Self::Reconciling)
                | (
                    Self::Reconciling,
                    Self::Succeeded | Self::RetrySafe | Self::NeedsUserAction
                )
                | (Self::RetrySafe, Self::Pending | Self::Cancelled)
                | (Self::NeedsUserAction, Self::Pending | Self::Cancelled)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteIssueLink {
    pub id: String,
    pub local_issue_id: String,
    pub provider: String,
    pub account_id: String,
    pub repository_id: Option<String>,
    pub remote_id: String,
    pub remote_number: i64,
    pub remote_url: String,
    pub remote_updated_at: Option<String>,
    pub last_synced_at: Option<String>,
    pub field_ownership: String,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboxJob {
    pub id: String,
    pub local_issue_id: Option<String>,
    pub operation: String,
    pub payload_json: String,
    pub state: String,
    pub attempts: i64,
    pub idempotency_key: String,
    pub fingerprint: String,
    pub reconciliation_hint: Option<String>,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncCursor {
    pub id: String,
    pub provider: String,
    pub account_id: String,
    pub resource: String,
    pub scope: String,
    pub page_cursor: Option<String>,
    pub watermark: Option<String>,
    pub last_success_at: Option<String>,
    pub last_attempt_at: Option<String>,
    pub next_due_at: Option<String>,
    pub last_error: Option<String>,
    pub incomplete_reason: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConflict {
    pub id: String,
    pub local_issue_id: String,
    pub field: String,
    pub local_value: Option<String>,
    pub remote_value: Option<String>,
    pub remote_updated_at: Option<String>,
    pub resolution: Option<String>,
    pub resolved_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncCursorInput {
    pub page_cursor: Option<String>,
    pub watermark: Option<String>,
    pub last_success_at: Option<String>,
    pub last_attempt_at: Option<String>,
    pub next_due_at: Option<String>,
    pub last_error: Option<String>,
    pub incomplete_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboxInput {
    pub local_issue_id: Option<String>,
    pub operation: String,
    #[serde(default)]
    pub payload_json: String,
    pub idempotency_key: String,
    pub fingerprint: String,
    pub reconciliation_hint: Option<String>,
}

impl OutboxInput {
    pub fn validate(&self) -> CoreResult<()> {
        if self.operation.trim().is_empty() {
            return Err(CoreError::validation("operation", "Operation is required."));
        }
        if self.idempotency_key.trim().is_empty() || self.idempotency_key.len() > 200 {
            return Err(CoreError::validation(
                "idempotencyKey",
                "Idempotency key must be between 1 and 200 characters.",
            ));
        }
        if self.fingerprint.trim().is_empty() {
            return Err(CoreError::validation(
                "fingerprint",
                "A request fingerprint is required.",
            ));
        }
        Ok(())
    }
}
