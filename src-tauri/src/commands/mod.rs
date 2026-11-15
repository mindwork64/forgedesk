pub mod error;

use forgedesk_core::app::{Bootstrap, RestoreResult};
use forgedesk_core::backup::BackupInfo;
use forgedesk_core::domain::{
    ApiControl, ApiMonitorState, AppSettings, BatchIssueInput, CreateIssueInput, Cycle,
    CycleDetail, CycleInput, Document, DocumentInput, GitAvailability, GitDiffStat, GitLogEntry,
    GitOverview, Initiative, InitiativeDetail, InitiativeInput, IssueDetail, IssueQuery,
    IssueSummary, Label, Milestone, MilestoneInput, Project, ProjectDetail, ProjectInput,
    ProjectQuery, ProjectRef, RelationInputKind, Repository, RepositoryInput, SavedView,
    SavedViewInput, SettingsPatch, StatusSummary, UpdateIssueInput, WindowState, Workflow,
    Workspace,
};
use forgedesk_core::github_auth::{DeviceFlowPoll, DeviceFlowStart, GitHubConnection};
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

#[tauri::command]
pub fn list_saved_views(state: State<'_, AppState>) -> Result<Vec<SavedView>, CommandError> {
    state
        .with_service(|service| service.list_saved_views())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn create_saved_view(
    state: State<'_, AppState>,
    input: SavedViewInput,
) -> Result<SavedView, CommandError> {
    state
        .with_service(|service| service.create_saved_view(input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn update_saved_view(
    state: State<'_, AppState>,
    id: String,
    input: SavedViewInput,
) -> Result<SavedView, CommandError> {
    state
        .with_service(|service| service.update_saved_view(&id, input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn delete_saved_view(state: State<'_, AppState>, id: String) -> Result<(), CommandError> {
    state
        .with_service(|service| service.delete_saved_view(&id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_project_cards(
    state: State<'_, AppState>,
    query: ProjectQuery,
) -> Result<Vec<Project>, CommandError> {
    state
        .with_service(|service| service.list_project_cards(query))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_project(state: State<'_, AppState>, id: String) -> Result<ProjectDetail, CommandError> {
    state
        .with_service(|service| service.get_project(&id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn create_project(
    state: State<'_, AppState>,
    input: ProjectInput,
) -> Result<Project, CommandError> {
    state
        .with_service(|service| service.create_project(input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn update_project(
    state: State<'_, AppState>,
    id: String,
    input: ProjectInput,
) -> Result<Project, CommandError> {
    state
        .with_service(|service| service.update_project(&id, input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn archive_project(
    state: State<'_, AppState>,
    id: String,
    archived: bool,
) -> Result<Project, CommandError> {
    state
        .with_service(|service| service.archive_project(&id, archived))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn create_milestone(
    state: State<'_, AppState>,
    project_id: String,
    input: MilestoneInput,
) -> Result<Milestone, CommandError> {
    state
        .with_service(|service| service.create_milestone(&project_id, input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn update_milestone(
    state: State<'_, AppState>,
    id: String,
    input: MilestoneInput,
) -> Result<Milestone, CommandError> {
    state
        .with_service(|service| service.update_milestone(&id, input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn assign_milestone_issue(
    state: State<'_, AppState>,
    milestone_id: String,
    issue_id: String,
    assigned: bool,
) -> Result<(), CommandError> {
    state
        .with_service(|service| service.assign_milestone_issue(&milestone_id, &issue_id, assigned))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_initiatives(state: State<'_, AppState>) -> Result<Vec<Initiative>, CommandError> {
    state
        .with_service(|service| service.list_initiatives())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_initiative(
    state: State<'_, AppState>,
    id: String,
) -> Result<InitiativeDetail, CommandError> {
    state
        .with_service(|service| service.get_initiative(&id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn create_initiative(
    state: State<'_, AppState>,
    input: InitiativeInput,
) -> Result<Initiative, CommandError> {
    state
        .with_service(|service| service.create_initiative(input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn update_initiative(
    state: State<'_, AppState>,
    id: String,
    input: InitiativeInput,
) -> Result<Initiative, CommandError> {
    state
        .with_service(|service| service.update_initiative(&id, input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_cycles(state: State<'_, AppState>) -> Result<Vec<Cycle>, CommandError> {
    state
        .with_service(|service| service.list_cycles())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_cycle(state: State<'_, AppState>, id: String) -> Result<CycleDetail, CommandError> {
    state
        .with_service(|service| service.get_cycle(&id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn create_cycle(state: State<'_, AppState>, input: CycleInput) -> Result<Cycle, CommandError> {
    state
        .with_service(|service| service.create_cycle(input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn update_cycle(
    state: State<'_, AppState>,
    id: String,
    input: CycleInput,
) -> Result<Cycle, CommandError> {
    state
        .with_service(|service| service.update_cycle(&id, input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn assign_cycle_issue(
    state: State<'_, AppState>,
    cycle_id: String,
    issue_id: String,
    assigned: bool,
) -> Result<(), CommandError> {
    state
        .with_service(|service| service.assign_cycle_issue(&cycle_id, &issue_id, assigned))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_documents(state: State<'_, AppState>) -> Result<Vec<Document>, CommandError> {
    state
        .with_service(|service| service.list_documents())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_document(state: State<'_, AppState>, id: String) -> Result<Document, CommandError> {
    state
        .with_service(|service| service.get_document(&id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn create_document(
    state: State<'_, AppState>,
    input: DocumentInput,
) -> Result<Document, CommandError> {
    state
        .with_service(|service| service.create_document(input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn update_document(
    state: State<'_, AppState>,
    id: String,
    input: DocumentInput,
) -> Result<Document, CommandError> {
    state
        .with_service(|service| service.update_document(&id, input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn delete_document(state: State<'_, AppState>, id: String) -> Result<(), CommandError> {
    state
        .with_service(|service| service.delete_document(&id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn export_document(state: State<'_, AppState>, id: String) -> Result<String, CommandError> {
    state
        .with_service(|service| service.export_document(&id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn git_available() -> Result<GitAvailability, CommandError> {
    Ok(forgedesk_core::git::git_available())
}

#[tauri::command]
pub fn list_repositories(state: State<'_, AppState>) -> Result<Vec<Repository>, CommandError> {
    state
        .with_service(|service| service.list_repositories())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn add_local_repository(
    state: State<'_, AppState>,
    input: RepositoryInput,
) -> Result<Repository, CommandError> {
    state
        .with_service(|service| service.add_local_repository(input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn remove_repository(state: State<'_, AppState>, id: String) -> Result<(), CommandError> {
    state
        .with_service(|service| service.remove_repository(&id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_git_overview(
    state: State<'_, AppState>,
    id: String,
) -> Result<GitOverview, CommandError> {
    state
        .with_service(|service| service.git_overview(&id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn create_git_branch(
    state: State<'_, AppState>,
    id: String,
    branch: String,
) -> Result<GitOverview, CommandError> {
    state
        .with_service(|service| service.create_git_branch(&id, &branch))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn switch_git_branch(
    state: State<'_, AppState>,
    id: String,
    branch: String,
) -> Result<GitOverview, CommandError> {
    state
        .with_service(|service| service.switch_git_branch(&id, &branch))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_git_log(
    state: State<'_, AppState>,
    id: String,
    limit: usize,
) -> Result<Vec<GitLogEntry>, CommandError> {
    state
        .with_service(|service| service.git_log(&id, limit))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_git_diff(state: State<'_, AppState>, id: String) -> Result<GitDiffStat, CommandError> {
    state
        .with_service(|service| service.git_diff(&id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_api_monitor(state: State<'_, AppState>) -> Result<ApiMonitorState, CommandError> {
    state
        .with_service(|service| service.api_monitor())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn set_api_paused(
    state: State<'_, AppState>,
    paused: bool,
    reason: String,
) -> Result<ApiControl, CommandError> {
    state
        .with_service(|service| service.set_api_paused(paused, &reason))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn export_api_diagnostics(state: State<'_, AppState>) -> Result<String, CommandError> {
    state
        .with_service(|service| service.export_api_diagnostics())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_start_device_flow(
    state: State<'_, AppState>,
) -> Result<DeviceFlowStart, CommandError> {
    state
        .with_github(|github| github.start_device_flow())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_poll_device_flow(state: State<'_, AppState>) -> Result<DeviceFlowPoll, CommandError> {
    state
        .with_github(|github| github.poll_device_flow())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_connection(
    state: State<'_, AppState>,
) -> Result<Option<GitHubConnection>, CommandError> {
    state
        .with_github(|github| github.connection())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_refresh(state: State<'_, AppState>) -> Result<GitHubConnection, CommandError> {
    state
        .with_github(|github| github.refresh())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_disconnect(state: State<'_, AppState>) -> Result<(), CommandError> {
    state
        .with_github(|github| github.disconnect())
        .map_err(CommandError::from)
}
