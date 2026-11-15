use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use keyring::Entry;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{CoreError, CoreResult};

pub const DEFAULT_CLIENT_ID: &str = "Iv23liclPWLrC2OpaeXz";
const KEYRING_SERVICE: &str = "dev.forgedesk.app";
const KEYRING_USER: &str = "github-app-user";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const DEVICE_URL: &str = "https://github.com/login/device/code";
const USER_URL: &str = "https://api.github.com/user";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceFlowStart {
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: i64,
    pub interval: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubConnection {
    pub connected: bool,
    pub login: String,
    pub user_id: i64,
    pub scopes: String,
    pub expires_at: String,
    pub refresh_expires_at: String,
    pub storage: String,
    pub storage_warning: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceFlowPoll {
    pub status: String,
    pub interval: i64,
    pub connection: Option<GitHubConnection>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: i64,
    interval: i64,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    expires_in: Option<i64>,
    refresh_token: Option<String>,
    refresh_token_expires_in: Option<i64>,
    scope: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredToken {
    access_token: String,
    refresh_token: String,
    expires_at: i64,
    refresh_expires_at: i64,
    login: String,
    user_id: i64,
    scopes: String,
}

#[derive(Debug, Clone)]
struct PendingDevice {
    device_code: String,
    interval: i64,
    expires_at: i64,
}

struct Inner {
    pending: Option<PendingDevice>,
    token: Option<StoredToken>,
    storage: String,
    storage_warning: Option<String>,
}

trait CredentialStore: Send + Sync {
    fn load(&self) -> CoreResult<Option<String>>;
    fn save(&self, value: &str) -> CoreResult<()>;
    fn delete(&self) -> CoreResult<()>;
}

struct KeyringStore;

impl CredentialStore for KeyringStore {
    fn load(&self) -> CoreResult<Option<String>> {
        match entry()?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(CoreError::auth(
                "Secure credential store could not read the GitHub token.",
            )),
        }
    }

    fn save(&self, value: &str) -> CoreResult<()> {
        entry()?.set_password(value).map_err(|_| {
            CoreError::auth("Secure credential store could not save the GitHub token.")
        })
    }

    fn delete(&self) -> CoreResult<()> {
        match entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(CoreError::auth(
                "Secure credential store could not delete the GitHub token.",
            )),
        }
    }
}

pub struct GitHubAuth {
    client_id: String,
    http: Client,
    store: Box<dyn CredentialStore>,
    inner: Mutex<Inner>,
}

impl GitHubAuth {
    pub fn new() -> CoreResult<Self> {
        Self::with_store(Box::new(KeyringStore))
    }

    fn with_store(store: Box<dyn CredentialStore>) -> CoreResult<Self> {
        let client_id = std::env::var("FORGEDESK_GITHUB_CLIENT_ID")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_CLIENT_ID.to_owned());
        let http = Client::builder()
            .user_agent("ForgeDesk/0.1 (+https://github.com/mindwork64/forgedesk)")
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| CoreError::network(format!("HTTP client failed: {error}")))?;
        Ok(Self {
            client_id,
            http,
            store,
            inner: Mutex::new(Inner {
                pending: None,
                token: None,
                storage: "memory".into(),
                storage_warning: None,
            }),
        })
    }

    pub fn start_device_flow(&self) -> CoreResult<DeviceFlowStart> {
        let response = self
            .http
            .post(DEVICE_URL)
            .header("Accept", "application/json")
            .form(&[("client_id", self.client_id.as_str())])
            .send()
            .map_err(|error| CoreError::network(format!("GitHub is unreachable: {error}")))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|error| CoreError::network(format!("GitHub response failed: {error}")))?;
        let parsed: DeviceCodeResponse = serde_json::from_str(&body)
            .map_err(|_| CoreError::network("GitHub returned an invalid device response."))?;
        if let Some(error) = parsed.error {
            return Err(CoreError::auth(format!(
                "GitHub device flow failed: {error} {}",
                parsed.error_description.unwrap_or_default()
            )));
        }
        if !status.is_success() {
            return Err(CoreError::network(format!(
                "GitHub device flow returned HTTP {status}."
            )));
        }
        let now = unix_now();
        let interval = parsed.interval.max(5);
        let mut inner = self.lock()?;
        inner.pending = Some(PendingDevice {
            device_code: parsed.device_code,
            interval,
            expires_at: now + parsed.expires_in,
        });
        Ok(DeviceFlowStart {
            user_code: parsed.user_code,
            verification_uri: parsed.verification_uri,
            expires_in: parsed.expires_in,
            interval,
        })
    }

    pub fn poll_device_flow(&self) -> CoreResult<DeviceFlowPoll> {
        let (device_code, interval) = {
            let inner = self.lock()?;
            let pending = inner
                .pending
                .as_ref()
                .ok_or_else(|| CoreError::auth("Start GitHub device flow first."))?;
            if unix_now() >= pending.expires_at {
                return Err(CoreError::auth(
                    "The GitHub device code expired. Start again.",
                ));
            }
            (pending.device_code.clone(), pending.interval)
        };

        let response = self
            .http
            .post(TOKEN_URL)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("device_code", device_code.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .map_err(|error| CoreError::network(format!("GitHub is unreachable: {error}")))?;
        let body = response
            .text()
            .map_err(|error| CoreError::network(format!("GitHub response failed: {error}")))?;
        let parsed: TokenResponse = serde_json::from_str(&body)
            .map_err(|_| CoreError::network("GitHub returned an invalid token response."))?;
        if let Some(error) = parsed.error.as_deref() {
            return self.handle_token_error(error, interval);
        }
        let access_token = parsed
            .access_token
            .clone()
            .ok_or_else(|| CoreError::auth("GitHub returned no access token."))?;
        let connection = self.finish_token(parsed, access_token, None)?;
        Ok(DeviceFlowPoll {
            status: "success".into(),
            interval,
            connection: Some(connection),
        })
    }

    pub fn connection(&self) -> CoreResult<Option<GitHubConnection>> {
        self.hydrate_from_keyring()?;
        let needs_refresh = {
            let inner = self.lock()?;
            inner
                .token
                .as_ref()
                .map(|token| token.expires_at <= unix_now() + 60)
                .unwrap_or(false)
        };
        if needs_refresh {
            return self.refresh().map(Some);
        }
        let inner = self.lock()?;
        Ok(inner
            .token
            .as_ref()
            .map(|token| connection_from(token, &inner)))
    }

    pub fn refresh(&self) -> CoreResult<GitHubConnection> {
        self.hydrate_from_keyring()?;
        let (refresh_token, login, user_id, scopes) = {
            let inner = self.lock()?;
            let token = inner
                .token
                .as_ref()
                .ok_or_else(|| CoreError::auth("GitHub is not connected."))?;
            (
                token.refresh_token.clone(),
                token.login.clone(),
                token.user_id,
                token.scopes.clone(),
            )
        };
        let response = self
            .http
            .post(TOKEN_URL)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token.as_str()),
            ])
            .send()
            .map_err(|error| CoreError::network(format!("GitHub is unreachable: {error}")))?;
        let parsed: TokenResponse = response
            .json()
            .map_err(|error| CoreError::network(format!("GitHub refresh failed: {error}")))?;
        if let Some(error) = parsed.error.as_deref() {
            return Err(CoreError::auth(format!(
                "GitHub token refresh failed: {error} {}",
                parsed.error_description.unwrap_or_default()
            )));
        }
        let access_token = parsed
            .access_token
            .clone()
            .ok_or_else(|| CoreError::auth("GitHub refresh returned no access token."))?;
        self.finish_token(parsed, access_token, Some((login, user_id, scopes)))
    }

    /// Return a currently valid access token for provider requests. Refreshes
    /// proactively when the stored token is within 60 seconds of expiry.
    pub fn access_token(&self) -> CoreResult<String> {
        self.hydrate_from_keyring()?;
        let needs_refresh = {
            let inner = self.lock()?;
            inner
                .token
                .as_ref()
                .map(|token| token.expires_at <= unix_now() + 60)
                .unwrap_or(false)
        };
        if needs_refresh {
            self.refresh()?;
        }
        let inner = self.lock()?;
        inner
            .token
            .as_ref()
            .map(|token| token.access_token.clone())
            .ok_or_else(|| CoreError::auth("GitHub is not connected."))
    }

    pub fn disconnect(&self) -> CoreResult<()> {
        let mut inner = self.lock()?;
        if inner.storage == "keyring" {
            self.store.delete()?;
        }
        inner.token = None;
        inner.pending = None;
        inner.storage = "memory".into();
        inner.storage_warning = None;
        Ok(())
    }

    fn finish_token(
        &self,
        parsed: TokenResponse,
        access_token: String,
        existing_identity: Option<(String, i64, String)>,
    ) -> CoreResult<GitHubConnection> {
        let now = unix_now();
        let expires_at = now + parsed.expires_in.unwrap_or(28_800);
        let refresh_expires_at = now + parsed.refresh_token_expires_in.unwrap_or(15_552_000);
        let refresh_token = parsed
            .refresh_token
            .or_else(|| {
                self.lock()
                    .ok()?
                    .token
                    .as_ref()
                    .map(|token| token.refresh_token.clone())
            })
            .ok_or_else(|| CoreError::auth("GitHub returned no refresh token."))?;
        let (login, user_id, scopes) = match existing_identity {
            Some(identity) => identity,
            None => self.fetch_identity(&access_token)?,
        };
        let token = StoredToken {
            access_token,
            refresh_token,
            expires_at,
            refresh_expires_at,
            login,
            user_id,
            scopes: parsed.scope.unwrap_or(scopes),
        };
        self.store_token(&token)?;
        let inner = self.lock()?;
        Ok(connection_from(&token, &inner))
    }

    fn fetch_identity(&self, access_token: &str) -> CoreResult<(String, i64, String)> {
        let response = self
            .http
            .get(USER_URL)
            .header("Accept", "application/vnd.github+json")
            .bearer_auth(access_token)
            .send()
            .map_err(|error| CoreError::network(format!("GitHub user request failed: {error}")))?;
        let status = response.status();
        let body: Value = response
            .json()
            .map_err(|error| CoreError::network(format!("GitHub user response failed: {error}")))?;
        if !status.is_success() {
            return Err(CoreError::auth(format!(
                "GitHub user request returned HTTP {status}."
            )));
        }
        let login = body
            .get("login")
            .and_then(Value::as_str)
            .ok_or_else(|| CoreError::auth("GitHub user response has no login."))?
            .to_owned();
        let user_id = body
            .get("id")
            .and_then(Value::as_i64)
            .ok_or_else(|| CoreError::auth("GitHub user response has no id."))?;
        Ok((login, user_id, String::new()))
    }

    fn store_token(&self, token: &StoredToken) -> CoreResult<()> {
        let serialized = serde_json::to_string(token)
            .map_err(|error| CoreError::internal(format!("Token encoding failed: {error}")))?;
        let mut inner = self.lock()?;
        inner.token = Some(token.clone());
        match self.store.save(&serialized) {
            Ok(()) => {
                inner.storage = "keyring".into();
                inner.storage_warning = None;
            }
            Err(error) => {
                inner.storage = "memory".into();
                inner.storage_warning = Some(format!(
                    "{}. The token is kept only for this session.",
                    error.user_message()
                ));
            }
        }
        Ok(())
    }

    fn hydrate_from_keyring(&self) -> CoreResult<()> {
        let mut inner = self.lock()?;
        if inner.token.is_some() {
            return Ok(());
        }
        let Some(serialized) = self.store.load()? else {
            return Ok(());
        };
        let token = serde_json::from_str::<StoredToken>(&serialized).map_err(|_| {
            CoreError::auth("Saved GitHub credential is invalid; reconnect to GitHub.")
        })?;
        inner.token = Some(token);
        inner.storage = "keyring".into();
        Ok(())
    }

    fn handle_token_error(&self, error: &str, interval: i64) -> CoreResult<DeviceFlowPoll> {
        match error {
            "authorization_pending" => Ok(DeviceFlowPoll {
                status: "pending".into(),
                interval,
                connection: None,
            }),
            "slow_down" => {
                let mut inner = self.lock()?;
                if let Some(pending) = inner.pending.as_mut() {
                    pending.interval += 5;
                    return Ok(DeviceFlowPoll {
                        status: "slow_down".into(),
                        interval: pending.interval,
                        connection: None,
                    });
                }
                Err(CoreError::auth("GitHub device flow is no longer pending."))
            }
            "expired_token" => Err(CoreError::auth("The GitHub device code expired.")),
            "access_denied" => Err(CoreError::auth("GitHub authorization was denied.")),
            other => Err(CoreError::auth(format!(
                "GitHub authorization failed: {other}"
            ))),
        }
    }

    fn lock(&self) -> CoreResult<std::sync::MutexGuard<'_, Inner>> {
        self.inner
            .lock()
            .map_err(|_| CoreError::internal("GitHub auth state is unavailable."))
    }
}

fn entry() -> CoreResult<Entry> {
    Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|error| CoreError::auth(format!("Secure credential store unavailable: {error}")))
}

fn connection_from(token: &StoredToken, inner: &Inner) -> GitHubConnection {
    GitHubConnection {
        connected: true,
        login: token.login.clone(),
        user_id: token.user_id,
        scopes: token.scopes.clone(),
        expires_at: format_timestamp(token.expires_at),
        refresh_expires_at: format_timestamp(token.refresh_expires_at),
        storage: inner.storage.clone(),
        storage_warning: inner.storage_warning.clone(),
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn format_timestamp(value: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp(value, 0)
        .map(|date| date.to_rfc3339())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[derive(Clone, Default)]
    struct FakeStore(Arc<Mutex<FakeState>>);

    #[derive(Default)]
    struct FakeState {
        value: Option<String>,
        delete_error: bool,
        entry_error: bool,
        load_entry_error: bool,
        load_error: bool,
        loads: usize,
        deletes: usize,
    }

    impl CredentialStore for FakeStore {
        fn load(&self) -> CoreResult<Option<String>> {
            let mut state = self.0.lock().expect("fake");
            state.loads += 1;
            if state.load_entry_error {
                return Err(CoreError::auth("keyring entry unavailable"));
            }
            if state.load_error {
                return Err(CoreError::auth("keyring unavailable"));
            }
            Ok(state.value.clone())
        }

        fn save(&self, value: &str) -> CoreResult<()> {
            self.0.lock().expect("fake").value = Some(value.to_owned());
            Ok(())
        }

        fn delete(&self) -> CoreResult<()> {
            let mut state = self.0.lock().expect("fake");
            state.deletes += 1;
            if state.entry_error {
                return Err(CoreError::auth("entry failed"));
            }
            if state.delete_error {
                return Err(CoreError::auth("delete failed"));
            }
            state.value = None;
            Ok(())
        }
    }

    fn stored_token() -> StoredToken {
        StoredToken {
            access_token: "access".into(),
            refresh_token: "refresh".into(),
            expires_at: 1,
            refresh_expires_at: 2,
            login: "user".into(),
            user_id: 1,
            scopes: String::new(),
        }
    }

    fn pending() -> PendingDevice {
        PendingDevice {
            device_code: "device".into(),
            interval: 5,
            expires_at: 999,
        }
    }

    fn auth_with(store: FakeStore) -> GitHubAuth {
        GitHubAuth::with_store(Box::new(store)).expect("auth")
    }

    fn seed(auth: &GitHubAuth, storage: &str) {
        let mut inner = auth.inner.lock().expect("inner");
        inner.token = Some(stored_token());
        inner.pending = Some(pending());
        inner.storage = storage.into();
        inner.storage_warning = Some("warning".into());
    }

    fn stored_json() -> String {
        serde_json::to_string(&stored_token()).expect("token json")
    }

    #[test]
    fn hydrate_absent_credentials_is_not_an_error() {
        let store = FakeStore::default();
        let auth = auth_with(store);
        auth.hydrate_from_keyring().expect("absent");
        assert!(auth.inner.lock().expect("inner").token.is_none());
    }

    #[test]
    fn hydrate_restores_valid_token() {
        let store = FakeStore::default();
        store.0.lock().expect("fake").value = Some(stored_json());
        let auth = auth_with(store);
        auth.hydrate_from_keyring().expect("restore");
        let inner = auth.inner.lock().expect("inner");
        assert_eq!(
            inner.token.as_ref().map(|token| token.login.as_str()),
            Some("user")
        );
        assert_eq!(inner.storage, "keyring");
        assert!(inner.storage_warning.is_none());
    }

    #[test]
    fn hydrate_read_error_is_propagated_without_clearing_state() {
        let store = FakeStore::default();
        store.0.lock().expect("fake").load_error = true;
        let auth = auth_with(store);
        let mut inner = auth.inner.lock().expect("inner");
        inner.pending = Some(pending());
        inner.storage_warning = Some("warning".into());
        drop(inner);

        let error = auth.hydrate_from_keyring().expect_err("read error");
        assert_eq!(error.category(), crate::error::ErrorCategory::Auth);
        let inner = auth.inner.lock().expect("inner");
        assert!(inner.token.is_none());
        assert!(inner.pending.is_some());
        assert!(inner.storage_warning.is_some());
    }

    #[test]
    fn hydrate_entry_creation_error_is_propagated() {
        let store = FakeStore::default();
        store.0.lock().expect("fake").load_entry_error = true;
        let auth = auth_with(store);
        auth.inner.lock().expect("inner").pending = Some(pending());

        let error = auth.hydrate_from_keyring().expect_err("entry error");
        assert_eq!(error.category(), crate::error::ErrorCategory::Auth);
        assert!(auth.inner.lock().expect("inner").pending.is_some());
    }

    #[test]
    fn hydrate_invalid_json_is_safe_error_without_clearing_state() {
        let store = FakeStore::default();
        store.0.lock().expect("fake").value = Some("not-json-secret".into());
        let auth = auth_with(store);
        let mut inner = auth.inner.lock().expect("inner");
        inner.pending = Some(pending());
        inner.storage_warning = Some("warning".into());
        drop(inner);

        let error = auth.hydrate_from_keyring().expect_err("invalid json");
        assert_eq!(error.category(), crate::error::ErrorCategory::Auth);
        assert!(!error.user_message().contains("not-json-secret"));
        let inner = auth.inner.lock().expect("inner");
        assert!(inner.token.is_none());
        assert!(inner.pending.is_some());
        assert!(inner.storage_warning.is_some());
    }

    #[test]
    fn hydrate_memory_token_does_not_touch_keyring() {
        let store = FakeStore::default();
        store.0.lock().expect("fake").load_error = true;
        let auth = auth_with(store.clone());
        auth.inner.lock().expect("inner").token = Some(stored_token());
        auth.hydrate_from_keyring().expect("memory token");
        assert_eq!(store.0.lock().expect("fake").loads, 0);
    }

    #[test]
    fn disconnect_deletes_credential_and_clears_all_state() {
        let store = FakeStore::default();
        store.0.lock().expect("fake").value = Some("secret".into());
        let auth = auth_with(store.clone());
        seed(&auth, "keyring");

        auth.disconnect().expect("disconnect");

        let fake = store.0.lock().expect("fake");
        assert!(fake.value.is_none());
        assert_eq!(fake.deletes, 1);
        drop(fake);
        let inner = auth.inner.lock().expect("inner");
        assert!(inner.token.is_none());
        assert!(inner.pending.is_none());
        assert_eq!(inner.storage, "memory");
        assert!(inner.storage_warning.is_none());
    }

    #[test]
    fn disconnect_without_stored_entry_is_idempotent() {
        let store = FakeStore::default();
        let auth = auth_with(store.clone());
        seed(&auth, "keyring");

        auth.disconnect().expect("first disconnect");
        auth.disconnect().expect("second disconnect");

        let fake = store.0.lock().expect("fake");
        assert!(fake.value.is_none());
        assert_eq!(fake.deletes, 1, "memory-only repeat must not touch keyring");
    }

    #[test]
    fn disconnect_delete_error_keeps_consistent_state() {
        let store = FakeStore::default();
        store.0.lock().expect("fake").delete_error = true;
        store.0.lock().expect("fake").value = Some("secret".into());
        let auth = auth_with(store);
        seed(&auth, "keyring");

        let error = auth.disconnect().expect_err("delete must fail");
        assert_eq!(error.category(), crate::error::ErrorCategory::Auth);
        let inner = auth.inner.lock().expect("inner");
        assert!(inner.token.is_some());
        assert!(inner.pending.is_some());
        assert_eq!(inner.storage, "keyring");
        assert!(inner.storage_warning.is_some());
    }

    #[test]
    fn disconnect_entry_failure_is_reported_and_keeps_state() {
        let store = FakeStore::default();
        store.0.lock().expect("fake").entry_error = true;
        let auth = auth_with(store);
        seed(&auth, "keyring");

        let error = auth.disconnect().expect_err("entry failure must surface");
        assert_eq!(error.category(), crate::error::ErrorCategory::Auth);
        let inner = auth.inner.lock().expect("inner");
        assert!(inner.token.is_some());
        assert!(inner.pending.is_some());
        assert_eq!(inner.storage, "keyring");
    }

    #[test]
    fn memory_only_disconnect_does_not_require_keyring() {
        let store = FakeStore::default();
        store.0.lock().expect("fake").delete_error = true;
        let auth = auth_with(store.clone());
        seed(&auth, "memory");

        auth.disconnect().expect("memory disconnect");

        assert_eq!(store.0.lock().expect("fake").deletes, 0);
        let inner = auth.inner.lock().expect("inner");
        assert!(inner.token.is_none());
        assert!(inner.pending.is_none());
        assert!(inner.storage_warning.is_none());
    }

    #[test]
    fn token_response_parses_and_keeps_refresh() {
        let parsed: TokenResponse = serde_json::from_str(
            r#"{"access_token":"a","token_type":"bearer","expires_in":28800,"refresh_token":"r","refresh_token_expires_in":15552000,"scope":""}"#,
        )
        .expect("token");
        assert_eq!(parsed.access_token.as_deref(), Some("a"));
        assert_eq!(parsed.refresh_token.as_deref(), Some("r"));
        assert_eq!(parsed.expires_in, Some(28800));
    }

    #[test]
    fn device_error_payload_parses_without_secret() {
        let parsed: DeviceCodeResponse = serde_json::from_str(
            r#"{"error":"device_flow_disabled","error_description":"Device flow is disabled"}"#,
        )
        .expect("error");
        assert_eq!(parsed.error.as_deref(), Some("device_flow_disabled"));
        let body = serde_json::json!({"error":"authorization_pending"});
        assert_eq!(body["error"], "authorization_pending");
    }

    #[test]
    fn formatted_timestamp_is_rfc3339() {
        assert!(format_timestamp(1_700_000_000).contains('T'));
    }
}
