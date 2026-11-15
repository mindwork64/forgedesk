use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub trigger_event: String,
    pub action_type: String,
    pub action_value: String,
    pub last_run_at: Option<String>,
    pub last_result: Option<String>,
    pub run_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRun {
    pub id: String,
    pub rule_id: String,
    pub event_id: String,
    pub status: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationEvent {
    pub event_id: String,
    pub entity_id: String,
    pub entity_key: String,
    pub entity_title: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationInput {
    pub name: String,
    pub trigger_event: String,
    pub action_value: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl AutomationInput {
    pub fn validate(&self) -> CoreResult<()> {
        if self.name.trim().is_empty() || self.name.len() > 120 {
            return Err(CoreError::validation(
                "name",
                "Automation name must be between 1 and 120 characters.",
            ));
        }
        if !matches!(
            self.trigger_event.as_str(),
            "issue_created" | "status_changed"
        ) {
            return Err(CoreError::validation(
                "triggerEvent",
                "Unsupported trigger.",
            ));
        }
        if self.action_value.trim().is_empty() {
            return Err(CoreError::validation(
                "actionValue",
                "Label name is required.",
            ));
        }
        Ok(())
    }
}
