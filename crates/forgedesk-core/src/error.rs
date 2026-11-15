use serde::Serialize;

/// Stable, machine-readable error categories exposed to the UI.
///
/// These map onto the categories required by `02_TECHNICAL_SPEC.md` §9. New
/// variants may be added as later stages land; existing codes must not change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    Validation,
    LocalDb,
    Backup,
    Io,
    Conflict,
    NotFound,
    Startup,
    Internal,
    Auth,
    Permission,
    Network,
    Provider,
    RateLimitPrimary,
    RateLimitSecondary,
    UnsupportedCapability,
}

impl ErrorCategory {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Validation => "validation",
            Self::LocalDb => "local_db",
            Self::Backup => "backup",
            Self::Io => "io",
            Self::Conflict => "conflict",
            Self::NotFound => "not_found",
            Self::Startup => "startup",
            Self::Internal => "internal",
            Self::Auth => "auth",
            Self::Permission => "permission",
            Self::Network => "network",
            Self::Provider => "provider",
            Self::RateLimitPrimary => "rate_limit_primary",
            Self::RateLimitSecondary => "rate_limit_secondary",
            Self::UnsupportedCapability => "unsupported_capability",
        }
    }
}

/// Internal error type used across the core. The Tauri command layer converts
/// this into [`WireError`] so no raw SQL, paths or secrets reach the UI by
/// accident.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("{message}")]
    Validation {
        message: String,
        field: Option<String>,
    },

    #[error("local database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("file system error: {0}")]
    Io(#[from] std::io::Error),

    #[error("backup error: {message}")]
    Backup { message: String },

    #[error("conflict: {message}")]
    Conflict { message: String },

    #[error("not found: {entity}")]
    NotFound { entity: String },

    #[error("startup error: {message}")]
    Startup { message: String },

    #[error("internal error: {message}")]
    Internal { message: String },

    #[error("authentication error: {message}")]
    Auth { message: String },

    #[error("network error: {message}")]
    Network { message: String },

    #[error("unsupported capability: {message}")]
    UnsupportedCapability { message: String },
}

impl CoreError {
    pub fn validation(field: &str, message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field: Some(field.to_owned()),
        }
    }

    pub fn validation_message(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field: None,
        }
    }

    pub fn backup(message: impl Into<String>) -> Self {
        Self::Backup {
            message: message.into(),
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    pub fn not_found(entity: impl Into<String>) -> Self {
        Self::NotFound {
            entity: entity.into(),
        }
    }

    pub fn startup(message: impl Into<String>) -> Self {
        Self::Startup {
            message: message.into(),
        }
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth {
            message: message.into(),
        }
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
        }
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::UnsupportedCapability {
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::Validation { .. } => ErrorCategory::Validation,
            Self::Database(_) => ErrorCategory::LocalDb,
            Self::Io(_) => ErrorCategory::Io,
            Self::Backup { .. } => ErrorCategory::Backup,
            Self::Conflict { .. } => ErrorCategory::Conflict,
            Self::NotFound { .. } => ErrorCategory::NotFound,
            Self::Startup { .. } => ErrorCategory::Startup,
            Self::Internal { .. } => ErrorCategory::Internal,
            Self::Auth { .. } => ErrorCategory::Auth,
            Self::Network { .. } => ErrorCategory::Network,
            Self::UnsupportedCapability { .. } => ErrorCategory::UnsupportedCapability,
        }
    }

    /// User-facing message. Deliberately avoids raw SQL and file paths.
    pub fn user_message(&self) -> String {
        match self {
            Self::Validation { message, .. } => message.clone(),
            Self::Database(_) => "The local database could not complete the operation.".to_owned(),
            Self::Io(_) => "A local file operation failed.".to_owned(),
            Self::Backup { message } => message.clone(),
            Self::Conflict { message } => message.clone(),
            Self::NotFound { entity } => format!("{entity} was not found."),
            Self::Startup { message } => message.clone(),
            Self::Internal { message } => message.clone(),
            Self::Auth { message } => message.clone(),
            Self::Network { message } => message.clone(),
            Self::UnsupportedCapability { message } => message.clone(),
        }
    }

    /// Sanitized technical detail for the diagnostics disclosure. Never include
    /// credentials; database and IO errors contain paths/SQL only.
    pub fn technical_detail(&self) -> Option<String> {
        match self {
            Self::Validation { .. } | Self::NotFound { .. } => None,
            Self::Database(e) => Some(e.to_string()),
            Self::Io(e) => Some(e.to_string()),
            Self::Backup { .. } | Self::Conflict { .. } | Self::Startup { .. } => {
                Some(self.to_string())
            }
            Self::Internal { .. } => None,
            Self::Auth { .. } | Self::Network { .. } | Self::UnsupportedCapability { .. } => {
                Some(self.to_string())
            }
        }
    }

    pub fn into_wire(self) -> WireError {
        WireError::from(&self)
    }
}

/// Error shape returned by every Tauri command.
#[derive(Debug, Clone, Serialize)]
pub struct WireError {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl From<&CoreError> for WireError {
    fn from(value: &CoreError) -> Self {
        let field = match value {
            CoreError::Validation { field, .. } => field.clone(),
            _ => None,
        };
        Self {
            code: value.category().code(),
            message: value.user_message(),
            field,
            detail: value.technical_detail(),
        }
    }
}

impl From<CoreError> for WireError {
    fn from(value: CoreError) -> Self {
        Self::from(&value)
    }
}

pub type CoreResult<T> = Result<T, CoreError>;
