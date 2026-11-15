use serde::Serialize;

use forgedesk_core::{CoreError, WireError};

/// Stable error payload for every command. `code` is the machine-readable
/// category; `message` is safe for direct display.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl CommandError {
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: "internal".to_owned(),
            message: message.into(),
            field: None,
            detail: None,
        }
    }
}

impl From<WireError> for CommandError {
    fn from(value: WireError) -> Self {
        Self {
            code: value.code.to_owned(),
            message: value.message,
            field: value.field,
            detail: value.detail,
        }
    }
}

impl From<CoreError> for CommandError {
    fn from(value: CoreError) -> Self {
        WireError::from(&value).into()
    }
}
