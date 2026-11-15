pub mod error;

use forgedesk_core::app::{Bootstrap, RestoreResult};
use forgedesk_core::backup::BackupInfo;
use forgedesk_core::domain::{AppSettings, SettingsPatch, WindowState, Workspace};
use tauri::{AppHandle, Manager, State};

use crate::state::AppState;

pub use error::CommandError;

#[tauri::command]
pub fn bootstrap(state: State<'_, AppState>) -> Result<Bootstrap, CommandError> {
    state.bootstrap().map_err(CommandError::from)
}

#[tauri::command]
pub fn create_workspace(
    state: State<'_, AppState>,
    name: String,
) -> Result<Workspace, CommandError> {
    state
        .with_service(|service| service.create_workspace(&name))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn save_settings(
    state: State<'_, AppState>,
    patch: SettingsPatch,
) -> Result<AppSettings, CommandError> {
    state
        .with_service(|service| service.update_settings(patch))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn complete_onboarding(state: State<'_, AppState>) -> Result<AppSettings, CommandError> {
    state
        .with_service(|service| service.complete_onboarding())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn save_window_state(
    state: State<'_, AppState>,
    window: WindowState,
) -> Result<AppSettings, CommandError> {
    state
        .with_service(|service| service.save_window_state(window))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_backups(state: State<'_, AppState>) -> Result<Vec<BackupInfo>, CommandError> {
    state
        .with_service(|service| service.list_backups())
        .map_err(CommandError::from)
}

/// VACUUM INTO can take long enough to be noticeable on a large workspace, so
/// backup work is moved off the UI thread.
#[tauri::command]
pub async fn create_backup(app: AppHandle) -> Result<BackupInfo, CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        state
            .with_service(|service| service.create_backup())
            .map_err(CommandError::from)
    })
    .await
    .map_err(|error| CommandError::internal(format!("Backup task failed: {error}")))?
}

#[tauri::command]
pub async fn restore_backup(app: AppHandle, id: String) -> Result<RestoreResult, CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        state
            .with_service_mut(|service| service.restore_backup(&id))
            .map_err(CommandError::from)
    })
    .await
    .map_err(|error| CommandError::internal(format!("Restore task failed: {error}")))?
}
