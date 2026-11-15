use std::path::PathBuf;
use std::sync::Mutex;

use forgedesk_core::app::Bootstrap;
use forgedesk_core::domain::WindowState;
use forgedesk_core::github_auth::GitHubAuth;
use forgedesk_core::{AppService, CoreError, WireError};

/// Shared shell state. The service is `None` only when startup failed; the
/// original sanitized error is retained so the UI can render a truthful error
/// surface instead of silently re-initializing or wiping data.
pub struct AppState {
    inner: Mutex<Inner>,
}

struct Inner {
    data_dir: PathBuf,
    service: Option<AppService>,
    github: Option<GitHubAuth>,
    startup_error: Option<WireError>,
}

impl AppState {
    pub fn initialize(data_dir: PathBuf, app_version: String) -> Self {
        let (service, startup_error) = match AppService::open(&data_dir, app_version.clone()) {
            Ok(service) => (Some(service), None),
            Err(error) => {
                tracing::error!(
                    code = error.category().code(),
                    error = %error,
                    "ForgeDesk could not open the local database"
                );
                (None, Some(WireError::from(&error)))
            }
        };

        Self {
            inner: Mutex::new(Inner {
                data_dir,
                service,
                github: None,
                startup_error,
            }),
        }
    }

    pub fn data_dir(&self) -> Result<PathBuf, WireError> {
        let inner = self.lock()?;
        Ok(inner.data_dir.clone())
    }

    pub fn bootstrap(&self) -> Result<Bootstrap, WireError> {
        let inner = self.lock()?;
        if let Some(error) = &inner.startup_error {
            return Err(error.clone());
        }
        let service = inner
            .service
            .as_ref()
            .ok_or_else(|| WireError::from(&CoreError::startup("ForgeDesk is not ready.")))?;
        service.bootstrap().map_err(|error| WireError::from(&error))
    }

    /// Best-effort bootstrap used for restoring window geometry. Never panics
    /// and never surfaces a startup error twice.
    pub fn try_bootstrap(&self) -> Option<Bootstrap> {
        let inner = self.inner.lock().ok()?;
        let service = inner.service.as_ref()?;
        service.bootstrap().ok()
    }

    pub fn with_service<T>(
        &self,
        f: impl FnOnce(&AppService) -> Result<T, CoreError>,
    ) -> Result<T, WireError> {
        let inner = self.lock()?;
        if let Some(error) = &inner.startup_error {
            return Err(error.clone());
        }
        let service = inner
            .service
            .as_ref()
            .ok_or_else(|| WireError::from(&CoreError::startup("ForgeDesk is not ready.")))?;
        f(service).map_err(|error| WireError::from(&error))
    }

    pub fn with_github<T>(
        &self,
        f: impl FnOnce(&GitHubAuth) -> Result<T, CoreError>,
    ) -> Result<T, WireError> {
        let mut inner = self.lock()?;
        if let Some(error) = &inner.startup_error {
            return Err(error.clone());
        }
        if inner.github.is_none() {
            inner.github = Some(GitHubAuth::new().map_err(WireError::from)?);
        }
        let github = inner
            .github
            .as_ref()
            .ok_or_else(|| WireError::from(&CoreError::startup("GitHub auth is not ready.")))?;
        f(github).map_err(WireError::from)
    }

    pub fn with_service_mut<T>(
        &self,
        f: impl FnOnce(&mut AppService) -> Result<T, CoreError>,
    ) -> Result<T, WireError> {
        let mut inner = self.lock()?;
        if let Some(error) = &inner.startup_error {
            return Err(error.clone());
        }
        let service = inner
            .service
            .as_mut()
            .ok_or_else(|| WireError::from(&CoreError::startup("ForgeDesk is not ready.")))?;
        f(service).map_err(|error| WireError::from(&error))
    }

    pub fn save_window_state(&self, window: WindowState) {
        if let Err(error) = self.with_service(|service| service.save_window_state(window)) {
            tracing::warn!(code = %error.code, "Could not persist window state");
        }
    }

    pub fn shutdown(&self) {
        if let Ok(inner) = self.inner.lock() {
            if let Some(service) = inner.service.as_ref() {
                service.shutdown();
            }
        }
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Inner>, WireError> {
        self.inner.lock().map_err(|_| {
            WireError::from(&CoreError::internal(
                "The application state lock is poisoned.",
            ))
        })
    }
}
