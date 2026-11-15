pub mod error;

use forgedesk_core::app::{Bootstrap, RestoreResult};
use forgedesk_core::backup::BackupInfo;
use forgedesk_core::domain::{
    AppSettings, BatchIssueInput, CreateIssueInput, IssueDetail, IssueQuery, IssueSummary, Label,
    ProjectRef, RelationInputKind, SettingsPatch, StatusSummary, UpdateIssueInput, WindowState,
    Workflow, Workspace,
};
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

#[tauri::command]
pub fn list_workflow(state: State<'_, AppState>) -> Result<Workflow, CommandError> {
    state
        .with_service(|service| service.list_workflow())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_issue_statuses(state: State<'_, AppState>) -> Result<Vec<StatusSummary>, CommandError> {
    state
        .with_service(|service| service.list_statuses())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_labels(state: State<'_, AppState>) -> Result<Vec<Label>, CommandError> {
    state
        .with_service(|service| service.list_labels())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<ProjectRef>, CommandError> {
    state
        .with_service(|service| service.list_projects())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_issues(
    state: State<'_, AppState>,
    query: IssueQuery,
) -> Result<Vec<IssueSummary>, CommandError> {
    state
        .with_service(|service| service.list_issues(query))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_issue(state: State<'_, AppState>, id: String) -> Result<IssueDetail, CommandError> {
    state
        .with_service(|service| service.get_issue(&id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn create_issue(
    state: State<'_, AppState>,
    input: CreateIssueInput,
) -> Result<IssueDetail, CommandError> {
    state
        .with_service(|service| service.create_issue(input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn update_issue(
    state: State<'_, AppState>,
    id: String,
    input: UpdateIssueInput,
) -> Result<IssueDetail, CommandError> {
    state
        .with_service(|service| service.update_issue(&id, input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn archive_issue(
    state: State<'_, AppState>,
    id: String,
    archived: bool,
) -> Result<IssueDetail, CommandError> {
    state
        .with_service(|service| service.archive_issue(&id, archived))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn batch_update_issues(
    state: State<'_, AppState>,
    ids: Vec<String>,
    input: BatchIssueInput,
) -> Result<Vec<IssueDetail>, CommandError> {
    state
        .with_service(|service| service.batch_update_issues(&ids, input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn duplicate_issue(
    state: State<'_, AppState>,
    id: String,
) -> Result<IssueDetail, CommandError> {
    state
        .with_service(|service| service.duplicate_issue(&id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn add_issue_comment(
    state: State<'_, AppState>,
    issue_id: String,
    body: String,
) -> Result<IssueDetail, CommandError> {
    state
        .with_service(|service| service.add_issue_comment(&issue_id, &body))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn add_issue_relation(
    state: State<'_, AppState>,
    issue_id: String,
    related_issue_id: String,
    kind: RelationInputKind,
) -> Result<IssueDetail, CommandError> {
    state
        .with_service(|service| service.add_issue_relation(&issue_id, &related_issue_id, kind))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn remove_issue_relation(
    state: State<'_, AppState>,
    issue_id: String,
    relation_id: String,
) -> Result<IssueDetail, CommandError> {
    state
        .with_service(|service| service.remove_issue_relation(&issue_id, &relation_id))
        .map_err(CommandError::from)
}
