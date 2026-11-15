#[cfg(feature = "ai-extension")]
use std::sync::{Mutex, OnceLock};

#[cfg(feature = "ai-extension")]
use forgedesk_ai::{AiConfig, AiProvider, AiService, ContextSelection};
use serde_json::Value;

use forgedesk_core::error::CoreResult;
use forgedesk_core::CoreError;

#[cfg(feature = "ai-extension")]
static SERVICE: OnceLock<Result<Mutex<AiService>, String>> = OnceLock::new();

#[cfg(feature = "ai-extension")]
fn service(data_dir: &std::path::Path) -> CoreResult<&'static Mutex<AiService>> {
    match SERVICE.get_or_init(|| {
        AiService::new(data_dir)
            .map(Mutex::new)
            .map_err(|error| error.to_string())
    }) {
        Ok(service) => Ok(service),
        Err(message) => Err(CoreError::startup(message.clone())),
    }
}

#[cfg(feature = "ai-extension")]
fn map_error(error: forgedesk_ai::AiError) -> CoreError {
    match error {
        forgedesk_ai::AiError::Disabled => {
            CoreError::unsupported("AI extension is disabled at runtime.")
        }
        forgedesk_ai::AiError::Auth => CoreError::auth("AI provider rejected the API key."),
        forgedesk_ai::AiError::Network => CoreError::network("AI provider request failed."),
        forgedesk_ai::AiError::Protocol => CoreError::network("AI provider response was invalid."),
        forgedesk_ai::AiError::NotConfigured => {
            CoreError::auth("AI provider API key is not configured.")
        }
        forgedesk_ai::AiError::Config => {
            CoreError::validation_message("AI configuration is invalid.")
        }
    }
}

#[cfg(feature = "ai-extension")]
pub fn dispatch(data_dir: &std::path::Path, action: &str, payload: Value) -> CoreResult<Value> {
    let service = service(data_dir)?;
    let guard = service
        .lock()
        .map_err(|_| CoreError::internal("AI extension state is unavailable."))?;
    match action {
        "status" => serde_json::to_value(guard.status().map_err(map_error)?)
            .map_err(|error| CoreError::internal(error.to_string())),
        "configure" => {
            let provider = match payload
                .get("provider")
                .and_then(Value::as_str)
                .unwrap_or("openai")
            {
                "anthropic" => AiProvider::Anthropic,
                _ => AiProvider::Openai,
            };
            let config = AiConfig {
                enabled: payload
                    .get("enabled")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                provider,
                base_url: payload
                    .get("baseUrl")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                model: payload
                    .get("model")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
            };
            let key = payload
                .get("apiKey")
                .and_then(Value::as_str)
                .map(str::to_owned);
            serde_json::to_value(guard.configure(config, key).map_err(map_error)?)
                .map_err(|error| CoreError::internal(error.to_string()))
        }
        "clear_key" => serde_json::to_value(guard.clear_key().map_err(map_error)?)
            .map_err(|error| CoreError::internal(error.to_string())),
        "generate" => {
            let context: ContextSelection =
                serde_json::from_value(payload.get("context").cloned().unwrap_or(Value::Null))
                    .map_err(|_| CoreError::validation("context", "AI context is invalid."))?;
            let instruction = payload
                .get("instruction")
                .and_then(Value::as_str)
                .unwrap_or_default();
            serde_json::to_value(guard.generate(context, instruction).map_err(map_error)?)
                .map_err(|error| CoreError::internal(error.to_string()))
        }
        _ => Err(CoreError::validation("action", "Unsupported AI action.")),
    }
}

#[cfg(not(feature = "ai-extension"))]
pub fn dispatch(_data_dir: &std::path::Path, _action: &str, _payload: Value) -> CoreResult<Value> {
    Err(CoreError::unsupported(
        "AI extension is not included in this build.",
    ))
}
