//! Optional AI extension. This crate is only linked when the desktop shell is built
//! with the `ai-extension` feature. Core never depends on it.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

pub mod mcp;

const KEYRING_SERVICE: &str = "dev.forgedesk.ai";
const KEYRING_USER: &str = "provider-api-key";

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("AI extension is disabled.")]
    Disabled,
    #[error("AI provider is not configured.")]
    NotConfigured,
    #[error("AI provider rejected the request.")]
    Auth,
    #[error("AI request failed.")]
    Network,
    #[error("AI response is invalid.")]
    Protocol,
    #[error("AI configuration could not be stored.")]
    Config,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    Openai,
    Anthropic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    #[serde(default)]
    pub enabled: bool,
    pub provider: AiProvider,
    pub base_url: String,
    pub model: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: AiProvider::Openai,
            base_url: "https://api.openai.com/v1".into(),
            model: "gpt-4o-mini".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiStatus {
    pub feature_enabled: bool,
    pub runtime_enabled: bool,
    pub provider: AiProvider,
    pub base_url: String,
    pub model: String,
    pub has_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextSelection {
    pub issue_key: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub project: Option<String>,
    #[serde(default)]
    pub include_description: bool,
}

impl ContextSelection {
    pub fn prompt(&self) -> String {
        let mut prompt = format!(
            "You are assisting with ForgeDesk issue {} — {}.\nStatus: {}.\n",
            self.issue_key, self.title, self.status
        );
        if let Some(project) = &self.project {
            prompt.push_str(&format!("Project: {project}.\n"));
        }
        if self.include_description && !self.description.trim().is_empty() {
            prompt.push_str("\nIssue description:\n");
            prompt.push_str(self.description.trim());
            prompt.push('\n');
        }
        prompt
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSuggestion {
    pub text: String,
    pub context_preview: String,
}

pub struct AiService {
    config_path: PathBuf,
    config: Mutex<AiConfig>,
    client: Client,
}

impl AiService {
    pub fn new(data_dir: &Path) -> Result<Self, AiError> {
        let config_path = data_dir.join("ai-config.json");
        let config = fs::read_to_string(&config_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<AiConfig>(&raw).ok())
            .unwrap_or_default();
        let client = Client::builder()
            .user_agent("ForgeDesk/0.1 AI extension")
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|_| AiError::Network)?;
        Ok(Self {
            config_path,
            config: Mutex::new(config),
            client,
        })
    }

    pub fn status(&self) -> Result<AiStatus, AiError> {
        let config = self.config.lock().map_err(|_| AiError::Config)?.clone();
        let has_key = if config.enabled {
            keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
                .ok()
                .and_then(|entry| entry.get_password().ok())
                .is_some()
        } else {
            false
        };
        Ok(AiStatus {
            feature_enabled: true,
            runtime_enabled: config.enabled,
            provider: config.provider,
            base_url: config.base_url.clone(),
            model: config.model.clone(),
            has_key,
        })
    }

    pub fn configure(
        &self,
        config: AiConfig,
        api_key: Option<String>,
    ) -> Result<AiStatus, AiError> {
        if config.base_url.trim().is_empty() || config.model.trim().is_empty() {
            return Err(AiError::Config);
        }
        if let Some(key) = api_key.filter(|key| !key.trim().is_empty()) {
            keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
                .and_then(|entry| entry.set_password(key.trim()))
                .map_err(|_| AiError::Auth)?;
        }
        fs::write(
            &self.config_path,
            serde_json::to_vec_pretty(&config).map_err(|_| AiError::Config)?,
        )
        .map_err(|_| AiError::Config)?;
        *self.config.lock().map_err(|_| AiError::Config)? = config;
        self.status()
    }

    pub fn clear_key(&self) -> Result<AiStatus, AiError> {
        let _ = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
            .and_then(|entry| entry.delete_credential());
        self.status()
    }

    pub fn generate(
        &self,
        context: ContextSelection,
        instruction: &str,
    ) -> Result<AiSuggestion, AiError> {
        let config = self.config.lock().map_err(|_| AiError::Config)?.clone();
        if !config.enabled {
            return Err(AiError::Disabled);
        }
        let key = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
            .and_then(|entry| entry.get_password())
            .map_err(|_| AiError::NotConfigured)?;
        let preview = context.prompt();
        let user = format!("{preview}\nUser request:\n{}", instruction.trim());
        let text = match config.provider {
            AiProvider::Openai => self.openai(&config, &key, &user)?,
            AiProvider::Anthropic => self.anthropic(&config, &key, &user)?,
        };
        Ok(AiSuggestion {
            text,
            context_preview: preview,
        })
    }

    fn openai(&self, config: &AiConfig, key: &str, user: &str) -> Result<String, AiError> {
        let response = self
            .client
            .post(format!("{}/chat/completions", config.base_url.trim_end_matches('/')))
            .bearer_auth(key)
            .json(&serde_json::json!({
                "model": config.model,
                "messages": [
                    {"role": "system", "content": "Return concise, actionable suggestions. Never claim changes were applied."},
                    {"role": "user", "content": user}
                ]
            }))
            .send()
            .map_err(|_| AiError::Network)?;
        if !response.status().is_success() {
            return Err(AiError::Auth);
        }
        let body: serde_json::Value = response.json().map_err(|_| AiError::Protocol)?;
        body.pointer("/choices/0/message/content")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .ok_or(AiError::Protocol)
    }

    fn anthropic(&self, config: &AiConfig, key: &str, user: &str) -> Result<String, AiError> {
        let response = self
            .client
            .post(format!(
                "{}/v1/messages",
                config.base_url.trim_end_matches('/')
            ))
            .header("x-api-key", key)
            .header("anthropic-version", "2023-06-01")
            .json(&serde_json::json!({
                "model": config.model,
                "max_tokens": 1024,
                "messages": [{"role": "user", "content": user}]
            }))
            .send()
            .map_err(|_| AiError::Network)?;
        if !response.status().is_success() {
            return Err(AiError::Auth);
        }
        let body: serde_json::Value = response.json().map_err(|_| AiError::Protocol)?;
        body.pointer("/content/0/text")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .ok_or(AiError::Protocol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn context_respects_selection() {
        let context = ContextSelection {
            issue_key: "FD-1".into(),
            title: "Title".into(),
            description: "Secret body".into(),
            status: "Todo".into(),
            project: Some("Core".into()),
            include_description: false,
        };
        let prompt = context.prompt();
        assert!(prompt.contains("FD-1"));
        assert!(!prompt.contains("Secret body"));
    }

    #[test]
    fn provider_http_contracts_parse_responses() {
        for (provider, path, response) in [
            (
                AiProvider::Openai,
                "chat/completions",
                r#"{"choices":[{"message":{"content":"openai text"}}]}"#,
            ),
            (
                AiProvider::Anthropic,
                "v1/messages",
                r#"{"content":[{"text":"anthropic text"}]}"#,
            ),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
            let address = listener.local_addr().expect("address");
            let handler = thread::spawn(move || {
                let (mut stream, _) = listener.accept().expect("accept");
                let mut request = [0u8; 4096];
                let read = stream.read(&mut request).expect("read");
                let request = String::from_utf8_lossy(&request[..read]).to_string();
                assert!(request.contains(path));
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    response.len()
                );
                stream.write_all(head.as_bytes()).expect("head");
                stream.write_all(response.as_bytes()).expect("body");
            });
            let dir = tempfile::tempdir().expect("tempdir");
            let service = AiService::new(dir.path()).expect("service");
            let config = AiConfig {
                enabled: true,
                provider,
                base_url: format!("http://{address}"),
                model: "test".into(),
            };
            let result = match provider {
                AiProvider::Openai => service.openai(&config, "key", "prompt"),
                AiProvider::Anthropic => service.anthropic(&config, "key", "prompt"),
            }
            .expect("response");
            assert!(result.contains("text"));
            handler.join().expect("join");
        }
    }

    #[test]
    fn disabled_service_does_not_call_provider() {
        let dir = tempfile::tempdir().expect("tempdir");
        let service = AiService::new(dir.path()).expect("service");
        assert_eq!(
            service
                .generate(
                    ContextSelection {
                        issue_key: "FD-1".into(),
                        title: "T".into(),
                        description: String::new(),
                        status: "Todo".into(),
                        project: None,
                        include_description: false,
                    },
                    "help",
                )
                .expect_err("disabled")
                .to_string(),
            "AI extension is disabled."
        );
    }
}
