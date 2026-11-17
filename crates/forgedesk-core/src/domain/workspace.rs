use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};

pub const WORKSPACE_NAME_MAX: usize = 80;
pub const DEFAULT_KEY_PREFIX: &str = "FD";
pub const ISSUE_KEY_PREFIX_MAX: usize = 10;

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
    /// Normalize and validate the prefix used for newly allocated issue keys.
    /// Existing issue keys are immutable and are never rewritten.
    pub fn validate_key_prefix(raw: &str) -> CoreResult<String> {
        let prefix = raw.trim().to_ascii_uppercase();
        let valid = (1..=ISSUE_KEY_PREFIX_MAX).contains(&prefix.len())
            && {
                let mut chars = prefix.chars();
                matches!(chars.next(), Some(first) if first.is_ascii_alphabetic())
                    && chars.all(|ch| ch.is_ascii_alphanumeric())
            };
        if !valid {
            return Err(CoreError::validation(
                "keyPrefix",
                format!(
                    "Issue key prefix must start with a letter and contain 1 to {ISSUE_KEY_PREFIX_MAX} ASCII letters or digits."
                ),
            ));
        }
        Ok(prefix)
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_and_normalizes_key_prefixes() {
        assert_eq!(Workspace::validate_key_prefix(" exhi42 ").unwrap(), "EXHI42");
        for invalid in ["", "1FD", "A-B", "TOO-LONG-PREFIX", "éx"] {
            assert!(Workspace::validate_key_prefix(invalid).is_err(), "{invalid}");
        }
    }
}
