use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};

pub const WORKSPACE_NAME_MAX: usize = 80;
pub const DEFAULT_KEY_PREFIX: &str = "FD";

/// The single local workspace of a v1 installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Workspace {
    pub fn validate_name(raw: &str) -> CoreResult<String> {
        let name = raw.trim();
        if name.is_empty() {
            return Err(CoreError::validation("name", "Workspace name is required."));
        }
        if name.chars().count() > WORKSPACE_NAME_MAX {
            return Err(CoreError::validation(
                "name",
                format!("Workspace name must be at most {WORKSPACE_NAME_MAX} characters."),
            ));
        }
        if name.chars().any(|c| c.is_control()) {
            return Err(CoreError::validation(
                "name",
                "Workspace name must not contain control characters.",
            ));
        }
        Ok(name.to_owned())
    }
}
