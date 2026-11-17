pub mod error;

use forgedesk_core::app::{Bootstrap, RestoreResult};
use forgedesk_core::backup::BackupInfo;
use forgedesk_core::domain::automation::{
    AutomationEvent, AutomationInput, AutomationRule, AutomationRun,
};
use forgedesk_core::domain::dashboard::Dashboard;
use forgedesk_core::domain::inbox::InboxEvent;
use forgedesk_core::domain::insights::InsightsSummary;
use forgedesk_core::domain::pull_link::PullLink;
use forgedesk_core::domain::{
    ApiMonitorState, AppSettings, BatchIssueInput, CreateIssueInput, Cycle, CycleDetail,
    CycleInput, Document, DocumentInput, GitAvailability, GitDiffStat, GitLogEntry, GitOverview,
    Initiative, InitiativeDetail, InitiativeInput, IssueDetail, IssueQuery, IssueSummary, Label,
    Milestone, MilestoneInput, Project, ProjectDetail, ProjectInput, ProjectQuery, ProjectRef,
    RelationInputKind, Repository, RepositoryInput, SavedView, SavedViewInput, SettingsPatch,
    StatusSummary, UpdateIssueInput, WindowState, Workflow, Workspace,
};
use forgedesk_core::github_actions::PullCapabilities;
use forgedesk_core::github_activity::CiOverview;
use forgedesk_core::github_auth::{DeviceFlowPoll, DeviceFlowStart, GitHubConnection};
use forgedesk_core::github_client::GitHubPull;
use forgedesk_core::github_client::GitHubReview;
use forgedesk_core::github_client::{GitHubCheckRun, GitHubSearchIssue};
use forgedesk_core::github_client::{GitHubMergeResult, GitHubPull as GitHubPullModel};
use forgedesk_core::github_pr::PullRequestDetail;
use forgedesk_core::github_sync::ImportResult;
use forgedesk_core::provider::{ProviderCapabilities, ProviderKind};
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
pub fn update_workspace_key_prefix(
    state: State<'_, AppState>,
    prefix: String,
) -> Result<Workspace, CommandError> {
    state
        .with_service(|service| service.update_workspace_key_prefix(&prefix))
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
pub fn get_repository(state: State<'_, AppState>, id: String) -> Result<Repository, CommandError> {
    state
        .with_service(|service| service.get_repository(&id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_project_repositories(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<Repository>, CommandError> {
    state
        .with_service(|service| service.list_project_repositories(&project_id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn link_repository(
    state: State<'_, AppState>,
    repository_id: String,
    project_id: String,
) -> Result<Repository, CommandError> {
    state
        .with_service(|service| service.link_repository(&repository_id, &project_id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn unlink_repository(
    state: State<'_, AppState>,
    repository_id: String,
    project_id: String,
) -> Result<Repository, CommandError> {
    state
        .with_service(|service| service.unlink_repository(&repository_id, &project_id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn attach_repository_checkout(
    state: State<'_, AppState>,
    repository_id: String,
    path: String,
) -> Result<Repository, CommandError> {
    state
        .with_service(|service| service.attach_repository_checkout(&repository_id, &path))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn detach_repository_checkout(
    state: State<'_, AppState>,
    checkout_id: String,
) -> Result<(), CommandError> {
    state
        .with_service(|service| service.detach_repository_checkout(&checkout_id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn select_repository(
    state: State<'_, AppState>,
    repository_id: Option<String>,
) -> Result<Option<Repository>, CommandError> {
    state
        .with_service(|service| service.select_repository(repository_id.as_deref()))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn selected_repository(state: State<'_, AppState>) -> Result<Option<Repository>, CommandError> {
    state
        .with_service(|service| service.selected_repository())
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

#[tauri::command]
pub fn github_discover_repositories(
    state: State<'_, AppState>,
    start_page: Option<u32>,
) -> Result<forgedesk_core::domain::RepositoryDiscoveryResult, CommandError> {
    let token = state.with_github(|github| github.access_token())?;
    state
        .with_service(|service| {
            service.discover_github_repositories(&token, start_page.unwrap_or(1))
        })
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_import_issues(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    target_project_id: Option<String>,
    start_page: Option<u32>,
) -> Result<ImportResult, CommandError> {
    let (token, login, user_id) = state.with_github(|github| {
        let token = github.access_token()?;
        let connection = github
            .connection()?
            .ok_or_else(|| forgedesk_core::CoreError::auth("GitHub is not connected."))?;
        Ok((token, connection.login, connection.user_id))
    })?;
    state
        .with_service(|service| {
            service.import_github_issues(
                &token,
                &login,
                user_id,
                &owner,
                &repo,
                target_project_id.as_deref(),
                start_page.unwrap_or(1),
            )
        })
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_dispatch_issue_outbox(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
) -> Result<forgedesk_core::github_sync::OutboxDispatchResult, CommandError> {
    let token = state.with_github(|github| github.access_token())?;
    state
        .with_service(|service| service.dispatch_github_issue_outbox(&token, &owner, &repo))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_sync_conflicts(
    state: State<'_, AppState>,
) -> Result<Vec<forgedesk_core::domain::sync::SyncConflict>, CommandError> {
    state
        .with_service(|service| service.list_github_sync_conflicts())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_resolve_sync_conflict(
    state: State<'_, AppState>,
    conflict_id: String,
    resolution: String,
) -> Result<forgedesk_core::domain::sync::SyncConflict, CommandError> {
    state
        .with_service(|service| service.resolve_github_sync_conflict(&conflict_id, &resolution))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_list_pulls(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    pull_state: String,
) -> Result<Vec<GitHubPull>, CommandError> {
    let token = state.with_github(|github| github.access_token())?;
    state
        .with_service(|service| {
            service
                .list_github_pulls(&token, &owner, &repo, &pull_state)
                .map(|(pulls, _)| pulls)
        })
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_pull_detail(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    number: i64,
) -> Result<PullRequestDetail, CommandError> {
    let token = state.with_github(|github| github.access_token())?;
    state
        .with_service(|service| service.github_pull_detail(&token, &owner, &repo, number))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_review_requested(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
) -> Result<Vec<GitHubSearchIssue>, CommandError> {
    let (token, login) = state.with_github(|github| {
        let token = github.access_token()?;
        let connection = github
            .connection()?
            .ok_or_else(|| forgedesk_core::CoreError::auth("GitHub is not connected."))?;
        Ok((token, connection.login))
    })?;
    state
        .with_service(|service| service.github_review_requested(&token, &owner, &repo, &login))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_ci_overview(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
) -> Result<CiOverview, CommandError> {
    let token = state.with_github(|github| github.access_token())?;
    state
        .with_service(|service| service.github_ci_overview(&token, &owner, &repo))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_commit_checks(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    reference: String,
) -> Result<Vec<GitHubCheckRun>, CommandError> {
    let token = state.with_github(|github| github.access_token())?;
    state
        .with_service(|service| service.github_commit_checks(&token, &owner, &repo, &reference))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_pull_link(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    number: i64,
) -> Result<Option<PullLink>, CommandError> {
    let account_id = state.with_github(|github| {
        let connection = github
            .connection()?
            .ok_or_else(|| forgedesk_core::CoreError::auth("GitHub is not connected."))?;
        Ok(format!("github:{}", connection.user_id))
    })?;
    state
        .with_service(|service| service.github_pull_link(&account_id, &owner, &repo, number))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_link_pull(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    number: i64,
    issue_key: String,
) -> Result<PullLink, CommandError> {
    let (token, account_id) = state.with_github(|github| {
        let token = github.access_token()?;
        let connection = github
            .connection()?
            .ok_or_else(|| forgedesk_core::CoreError::auth("GitHub is not connected."))?;
        Ok((token, format!("github:{}", connection.user_id)))
    })?;
    state
        .with_service(|service| {
            service.link_github_pull(&token, &account_id, &owner, &repo, number, &issue_key)
        })
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_unlink_pull(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    number: i64,
) -> Result<(), CommandError> {
    let account_id = state.with_github(|github| {
        let connection = github
            .connection()?
            .ok_or_else(|| forgedesk_core::CoreError::auth("GitHub is not connected."))?;
        Ok(format!("github:{}", connection.user_id))
    })?;
    state
        .with_service(|service| service.unlink_github_pull(&account_id, &owner, &repo, number))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_pull_capabilities(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
) -> Result<PullCapabilities, CommandError> {
    let token = state.with_github(|github| github.access_token())?;
    state
        .with_service(|service| service.github_pull_capabilities(&token, &owner, &repo))
        .map_err(CommandError::from)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn github_merge_pull(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    number: i64,
    method: String,
    expected_sha: Option<String>,
    confirmed: bool,
) -> Result<GitHubMergeResult, CommandError> {
    let token = state.with_github(|github| github.access_token())?;
    state
        .with_service(|service| {
            service.github_merge_pull(
                &token,
                &owner,
                &repo,
                number,
                &method,
                expected_sha.as_deref(),
                confirmed,
            )
        })
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_close_pull(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    number: i64,
    confirmed: bool,
) -> Result<GitHubPullModel, CommandError> {
    let token = state.with_github(|github| github.access_token())?;
    state
        .with_service(|service| service.github_close_pull(&token, &owner, &repo, number, confirmed))
        .map_err(CommandError::from)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn github_submit_review(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    number: i64,
    event: String,
    body: String,
    confirmed: bool,
) -> Result<GitHubReview, CommandError> {
    let (token, login) = state.with_github(|github| {
        let token = github.access_token()?;
        let connection = github
            .connection()?
            .ok_or_else(|| forgedesk_core::CoreError::auth("GitHub is not connected."))?;
        Ok((token, connection.login))
    })?;
    state
        .with_service(|service| {
            service.github_submit_review(
                &token, &owner, &repo, number, &event, &body, &login, confirmed,
            )
        })
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn inbox_list(
    state: State<'_, AppState>,
    filter: String,
) -> Result<Vec<InboxEvent>, CommandError> {
    state
        .with_service(|service| service.inbox(&filter))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn activity_list(state: State<'_, AppState>) -> Result<Vec<InboxEvent>, CommandError> {
    state
        .with_service(|service| service.activity())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn inbox_mark_read(
    state: State<'_, AppState>,
    event_id: String,
    read: bool,
) -> Result<(), CommandError> {
    state
        .with_service(|service| service.inbox_mark_read(&event_id, read))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn inbox_archive(state: State<'_, AppState>, event_id: String) -> Result<(), CommandError> {
    state
        .with_service(|service| service.inbox_archive(&event_id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn inbox_mark_all_read(state: State<'_, AppState>) -> Result<i64, CommandError> {
    state
        .with_service(|service| service.inbox_mark_all_read())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_insights(
    state: State<'_, AppState>,
    range_days: i64,
) -> Result<InsightsSummary, CommandError> {
    state
        .with_service(|service| service.insights(range_days))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn automation_list_rules(
    state: State<'_, AppState>,
) -> Result<Vec<AutomationRule>, CommandError> {
    state
        .with_service(|service| service.automation_rules())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn automation_create(
    state: State<'_, AppState>,
    input: AutomationInput,
) -> Result<AutomationRule, CommandError> {
    state
        .with_service(|service| service.create_automation(&input))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn automation_toggle(
    state: State<'_, AppState>,
    rule_id: String,
    enabled: bool,
) -> Result<AutomationRule, CommandError> {
    state
        .with_service(|service| service.toggle_automation(&rule_id, enabled))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn automation_delete(state: State<'_, AppState>, rule_id: String) -> Result<(), CommandError> {
    state
        .with_service(|service| service.delete_automation(&rule_id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn automation_preview(
    state: State<'_, AppState>,
    rule_id: String,
) -> Result<Vec<AutomationEvent>, CommandError> {
    state
        .with_service(|service| service.preview_automation(&rule_id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn automation_run(state: State<'_, AppState>, rule_id: String) -> Result<i64, CommandError> {
    state
        .with_service(|service| service.run_automation(&rule_id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn automation_runs(
    state: State<'_, AppState>,
    rule_id: String,
) -> Result<Vec<AutomationRun>, CommandError> {
    state
        .with_service(|service| service.automation_runs(&rule_id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn dashboard_get(state: State<'_, AppState>) -> Result<Dashboard, CommandError> {
    state
        .with_service(|service| service.dashboard())
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn dashboard_add_widget(
    state: State<'_, AppState>,
    kind: String,
    width: i64,
) -> Result<Dashboard, CommandError> {
    state
        .with_service(|service| service.dashboard_add_widget(&kind, width))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn dashboard_remove_widget(
    state: State<'_, AppState>,
    widget_id: String,
) -> Result<Dashboard, CommandError> {
    state
        .with_service(|service| service.dashboard_remove_widget(&widget_id))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn dashboard_update_width(
    state: State<'_, AppState>,
    widget_id: String,
    width: i64,
) -> Result<Dashboard, CommandError> {
    state
        .with_service(|service| service.dashboard_update_width(&widget_id, width))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn dashboard_reorder(
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<Dashboard, CommandError> {
    state
        .with_service(|service| service.dashboard_reorder(&ids))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn github_sync_pull_events(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
) -> Result<i64, CommandError> {
    let token = state.with_github(|github| github.access_token())?;
    state
        .with_service(|service| service.sync_github_pull_events(&token, &owner, &repo))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn provider_capabilities(provider: String) -> Result<ProviderCapabilities, CommandError> {
    Ok(forgedesk_core::provider::capabilities(ProviderKind::parse(
        &provider,
    )?))
}

#[tauri::command]
pub fn ai_dispatch(
    state: State<'_, AppState>,
    action: String,
    payload: serde_json::Value,
) -> Result<serde_json::Value, CommandError> {
    let data_dir = state.data_dir()?;
    crate::ai::dispatch(&data_dir, &action, payload).map_err(CommandError::from)
}
