use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::backup::{self, BackupInfo};
use crate::db::repositories::{
    api_budget, issues, labels as label_repo, planning, projects, repo_registry, saved_views,
    workflow,
};
use crate::db::Database;
use crate::domain::automation::{AutomationEvent, AutomationInput, AutomationRule, AutomationRun};
use crate::domain::dashboard::Dashboard;
use crate::domain::inbox::{InboxEvent, InboxFilter};
use crate::domain::insights::InsightsSummary;
use crate::domain::pull_link::PullLink;
use crate::domain::{
    ActivityFeedItem, ApiControl, ApiMonitorState, AppSettings, BatchIssueInput, CreateIssueInput,
    Cycle, CycleDetail, CycleInput, Document, DocumentInput, GitAvailability, GitDiffStat,
    GitLogEntry, GitOverview, Initiative, InitiativeDetail, InitiativeInput, IssueDetail,
    IssueQuery, IssueRelation, IssueSummary, Label, Milestone, MilestoneInput, Project,
    ProjectDetail, ProjectInput, ProjectQuery, ProjectRef, RelationInputKind, Repository,
    RepositoryDiscoveryResult, RepositoryInput, SavedView, SavedViewInput, SettingsPatch,
    StatusSummary, UpdateIssueInput, WindowState, Workflow, Workspace,
};
use crate::error::{CoreError, CoreResult};
use crate::github_actions::PullCapabilities;
use crate::github_activity::CiOverview;
use crate::github_client::GitHubSearchIssue;
use crate::github_client::{GitHubMergeResult, GitHubPull, GitHubReview};
use crate::github_pr::PullRequestDetail;
use crate::github_sync::ImportResult;

pub const DB_FILE_NAME: &str = "forgedesk.db";
pub const BACKUPS_DIR_NAME: &str = "backups";

/// One payload for the initial UI render. Keeping this in a single command
/// avoids a waterfall of IPC calls and inconsistent partial state.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub app_version: String,
    pub schema_version: i64,
    pub data_dir: String,
    pub db_path: String,
    pub backups_dir: String,
    pub workspace: Option<Workspace>,
    pub settings: AppSettings,
    pub backups: Vec<BackupInfo>,
    pub startup_warning: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub restored: BackupInfo,
    pub safety_backup: BackupInfo,
    pub bootstrap: Bootstrap,
}

/// Application service owning the local database. All UI-facing operations go
/// through this type; it never depends on Tauri.
pub struct AppService {
    db: Database,
    data_dir: PathBuf,
    db_path: PathBuf,
    backups_dir: PathBuf,
    app_version: String,
    startup_warning: Option<String>,
}

impl AppService {
    pub fn open(data_dir: impl AsRef<Path>, app_version: impl Into<String>) -> CoreResult<Self> {
        let data_dir = data_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join(DB_FILE_NAME);
        let backups_dir = data_dir.join(BACKUPS_DIR_NAME);
        let db = Database::open(&db_path)?;

        let mut service = Self {
            db,
            data_dir,
            db_path,
            backups_dir,
            app_version: app_version.into(),
            startup_warning: None,
        };
        service.run_startup_maintenance();
        Ok(service)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn bootstrap(&self) -> CoreResult<Bootstrap> {
        let workspace = self.db.workspace()?;
        let settings = self.db.settings()?;
        let backups = backup::list(&self.backups_dir)?;
        Ok(Bootstrap {
            app_version: self.app_version.clone(),
            schema_version: self.db.schema_version()?,
            data_dir: self.data_dir.to_string_lossy().into_owned(),
            db_path: self.db_path.to_string_lossy().into_owned(),
            backups_dir: self.backups_dir.to_string_lossy().into_owned(),
            workspace,
            settings,
            backups,
            startup_warning: self.startup_warning.clone(),
        })
    }

    pub fn create_workspace(&self, name: &str) -> CoreResult<Workspace> {
        self.db
            .with_transaction(|tx| crate::db::repositories::workspace::create_initial(tx, name))
    }

    pub fn complete_onboarding(&self) -> CoreResult<AppSettings> {
        self.update_settings(SettingsPatch {
            onboarding_completed: Some(true),
            ..Default::default()
        })
    }

    pub fn update_settings(&self, patch: SettingsPatch) -> CoreResult<AppSettings> {
        let mut settings = self.db.settings()?;
        settings.apply(patch)?;
        self.db.save_settings(&settings)?;
        Ok(settings)
    }

    pub fn save_window_state(&self, window: WindowState) -> CoreResult<AppSettings> {
        self.update_settings(SettingsPatch {
            window: Some(window),
            ..Default::default()
        })
    }

    pub fn create_backup(&self) -> CoreResult<BackupInfo> {
        let info = backup::create(&self.db, &self.backups_dir, &self.app_version)?;
        let retention = self
            .db
            .settings()
            .map(|settings| settings.backup_retention)
            .unwrap_or(20);
        let _ = backup::prune(&self.backups_dir, retention);
        Ok(info)
    }

    pub fn list_backups(&self) -> CoreResult<Vec<BackupInfo>> {
        backup::list(&self.backups_dir)
    }

    pub fn restore_backup(&mut self, id: &str) -> CoreResult<RestoreResult> {
        let restored = backup::validate(&self.backups_dir, id)?;
        let db_file = backup::backup_path(&self.backups_dir, id)?;

        // Never replace the live database without a consistent escape hatch.
        let safety_backup = backup::create(&self.db, &self.backups_dir, &self.app_version)?;

        self.db.replace_with(&db_file)?;
        self.startup_warning = None;

        Ok(RestoreResult {
            restored,
            safety_backup,
            bootstrap: self.bootstrap()?,
        })
    }

    pub fn shutdown(&self) {
        let _ = self.db.checkpoint();
        let _ = self.db.close();
    }

    pub fn list_workflow(&self) -> CoreResult<Workflow> {
        let workspace = self.require_workspace()?;
        self.db.with_conn(|conn| {
            workflow::load(conn, &workspace.id)?.ok_or_else(|| CoreError::not_found("Workflow"))
        })
    }

    pub fn list_statuses(&self) -> CoreResult<Vec<StatusSummary>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| workflow::statuses(conn, &workspace.id))
    }

    pub fn list_labels(&self) -> CoreResult<Vec<Label>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| label_repo::list(conn, &workspace.id))
    }

    pub fn list_projects(&self) -> CoreResult<Vec<ProjectRef>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| projects::list(conn, &workspace.id))
    }

    pub fn list_issues(&self, query: IssueQuery) -> CoreResult<Vec<IssueSummary>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| issues::list(conn, &workspace.id, &query))
    }

    pub fn get_issue(&self, issue_id: &str) -> CoreResult<IssueDetail> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| issues::get_detail(conn, &workspace.id, issue_id))
    }

    pub fn create_issue(&self, input: CreateIssueInput) -> CoreResult<IssueDetail> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|tx| {
            let id = issues::create(tx, &workspace.id, &input)?;
            issues::get_detail(tx, &workspace.id, &id)
        })
    }

    pub fn update_issue(&self, issue_id: &str, input: UpdateIssueInput) -> CoreResult<IssueDetail> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| issues::update(tx, &workspace.id, issue_id, &input))
    }

    pub fn archive_issue(&self, issue_id: &str, archived: bool) -> CoreResult<IssueDetail> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| issues::archive(tx, &workspace.id, issue_id, archived))
    }

    pub fn batch_update_issues(
        &self,
        issue_ids: &[String],
        input: BatchIssueInput,
    ) -> CoreResult<Vec<IssueDetail>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| issues::batch_update(tx, &workspace.id, issue_ids, &input))
    }

    pub fn duplicate_issue(&self, issue_id: &str) -> CoreResult<IssueDetail> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| issues::duplicate(tx, &workspace.id, issue_id))
    }

    pub fn add_issue_comment(&self, issue_id: &str, body: &str) -> CoreResult<IssueDetail> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| issues::add_comment(tx, &workspace.id, issue_id, body))
    }

    pub fn add_issue_relation(
        &self,
        issue_id: &str,
        related_issue_id: &str,
        kind: RelationInputKind,
    ) -> CoreResult<IssueDetail> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|tx| {
            issues::add_relation(tx, &workspace.id, issue_id, related_issue_id, kind)
        })
    }

    pub fn remove_issue_relation(
        &self,
        issue_id: &str,
        relation_id: &str,
    ) -> CoreResult<IssueDetail> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|tx| {
            issues::remove_relation(tx, &workspace.id, issue_id, relation_id)
        })
    }

    pub fn list_issue_relations(&self, issue_id: &str) -> CoreResult<Vec<IssueRelation>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| issues::list_relations(conn, &workspace.id, issue_id))
    }

    pub fn discover_github_repositories(
        &self,
        token: &str,
        start_page: u32,
    ) -> CoreResult<RepositoryDiscoveryResult> {
        let client = crate::github_client::GitHubClient::new(token)?;
        let workspace = self.require_workspace()?;
        let (remotes, has_more, pages_loaded, next_page) =
            client.list_repositories_bounded(start_page, 10)?;
        let mut items = Vec::new();
        for remote in remotes {
            let canonical = crate::db::repositories::repo_registry::normalize_remote(
                remote
                    .clone_url
                    .as_deref()
                    .or(Some(remote.html_url.as_str()))
                    .unwrap_or_default(),
            )
            .map(|value| value.canonical);
            let repository = self.db.with_transaction(|tx| {
                crate::db::repositories::repo_registry::upsert_remote(
                    tx,
                    &workspace.id,
                    &crate::db::repositories::repo_registry::RemoteRepositoryInput {
                        provider: "github".into(),
                        host: "github.com".into(),
                        external_id: Some(remote.id.to_string()),
                        owner: Some(remote.owner.login.clone()),
                        name: remote.name.clone(),
                        full_name: remote.full_name.clone(),
                        description: remote.description.clone(),
                        visibility: remote.visibility.clone().or_else(|| {
                            Some(if remote.private { "private" } else { "public" }.into())
                        }),
                        default_branch: remote.default_branch.clone(),
                        html_url: Some(remote.html_url.clone()),
                        clone_url: remote.clone_url.clone(),
                        canonical_remote_url: canonical,
                        is_archived: remote.archived,
                    },
                )
            })?;
            items.push(repository);
        }
        Ok(RepositoryDiscoveryResult {
            items,
            has_more,
            pages_loaded,
            next_page,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn import_github_issues(
        &self,
        token: &str,
        account_login: &str,
        user_id: i64,
        owner: &str,
        repo: &str,
        target_project_id: Option<&str>,
        start_page: u32,
    ) -> CoreResult<ImportResult> {
        let client = crate::github_client::GitHubClient::new(token)?;
        let fetched = crate::github_sync::fetch_import(&client, owner, repo, start_page)?;
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|tx| {
            crate::github_sync::apply_import(
                tx,
                &workspace.id,
                account_login,
                user_id,
                &fetched,
                target_project_id,
            )
        })
    }

    pub fn list_github_pulls(
        &self,
        token: &str,
        owner: &str,
        repo: &str,
        state: &str,
    ) -> CoreResult<(Vec<GitHubPull>, bool)> {
        crate::github_pr::list_pulls(token, owner, repo, state)
    }

    pub fn github_pull_detail(
        &self,
        token: &str,
        owner: &str,
        repo: &str,
        number: i64,
    ) -> CoreResult<PullRequestDetail> {
        crate::github_pr::pull_detail(token, owner, repo, number)
    }

    pub fn github_review_requested(
        &self,
        token: &str,
        owner: &str,
        repo: &str,
        login: &str,
    ) -> CoreResult<Vec<GitHubSearchIssue>> {
        crate::github_activity::review_requested(token, owner, repo, login)
    }

    pub fn github_ci_overview(
        &self,
        token: &str,
        owner: &str,
        repo: &str,
    ) -> CoreResult<CiOverview> {
        crate::github_activity::ci_overview(token, owner, repo)
    }

    pub fn github_commit_checks(
        &self,
        token: &str,
        owner: &str,
        repo: &str,
        reference: &str,
    ) -> CoreResult<Vec<crate::github_client::GitHubCheckRun>> {
        crate::github_activity::commit_checks(token, owner, repo, reference)
    }

    pub fn github_pull_link(
        &self,
        account_id: &str,
        owner: &str,
        repo: &str,
        number: i64,
    ) -> CoreResult<Option<PullLink>> {
        let workspace = self.require_workspace()?;
        self.db.with_conn(|conn| {
            crate::db::repositories::pull_links::get(
                conn,
                &workspace.id,
                "github",
                account_id,
                &format!("{owner}/{repo}"),
                number,
            )
        })
    }

    pub fn link_github_pull(
        &self,
        token: &str,
        account_id: &str,
        owner: &str,
        repo: &str,
        number: i64,
        issue_key: &str,
    ) -> CoreResult<PullLink> {
        let workspace = self.require_workspace()?;
        let pull = crate::github_client::GitHubClient::new(token)?.get_pull(owner, repo, number)?;
        self.db.with_transaction(|tx| {
            crate::db::repositories::pull_links::link(
                tx,
                &workspace.id,
                issue_key,
                "github",
                account_id,
                &format!("{owner}/{repo}"),
                number,
                &pull.html_url,
                &pull.title,
                &pull.state,
            )
        })
    }

    pub fn unlink_github_pull(
        &self,
        account_id: &str,
        owner: &str,
        repo: &str,
        number: i64,
    ) -> CoreResult<()> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|tx| {
            crate::db::repositories::pull_links::unlink(
                tx,
                &workspace.id,
                "github",
                account_id,
                &format!("{owner}/{repo}"),
                number,
            )
        })
    }

    pub fn github_pull_capabilities(
        &self,
        token: &str,
        owner: &str,
        repo: &str,
    ) -> CoreResult<PullCapabilities> {
        crate::github_actions::capabilities(token, owner, repo)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn github_merge_pull(
        &self,
        token: &str,
        owner: &str,
        repo: &str,
        number: i64,
        method: &str,
        expected_sha: Option<&str>,
        confirmed: bool,
    ) -> CoreResult<GitHubMergeResult> {
        let mut result = crate::github_actions::merge_pull(
            token,
            owner,
            repo,
            number,
            method,
            expected_sha,
            confirmed,
        )?;
        if result.merged {
            let workspace = self.require_workspace()?;
            let key = self.db.with_transaction(|conn| {
                crate::db::repositories::pull_links::complete_linked_issue(
                    conn,
                    &workspace.id,
                    "github",
                    &format!("{owner}/{repo}"),
                    number,
                )
            })?;
            result.completed_issue_key = key;
        }
        Ok(result)
    }

    pub fn sync_github_pull_events(&self, token: &str, owner: &str, repo: &str) -> CoreResult<i64> {
        let client = crate::github_client::GitHubClient::new(token)?;
        let mut pulls = client.list_pulls(owner, repo, "open")?.items;
        pulls.extend(client.list_pulls(owner, repo, "closed")?.items);
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|conn| {
            let mut changed = 0;
            for pull in &pulls {
                let state = if pull.merged_at.is_some() {
                    "merged"
                } else {
                    pull.state.as_str()
                };
                if crate::db::repositories::pull_links::sync_pull_state(
                    conn,
                    &workspace.id,
                    "github",
                    &format!("{owner}/{repo}"),
                    pull.number,
                    state,
                    &pull.title,
                    &pull.html_url,
                )? {
                    changed += 1;
                }
            }
            Ok(changed)
        })
    }

    pub fn github_close_pull(
        &self,
        token: &str,
        owner: &str,
        repo: &str,
        number: i64,
        confirmed: bool,
    ) -> CoreResult<GitHubPull> {
        crate::github_actions::close_pull(token, owner, repo, number, confirmed)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn github_submit_review(
        &self,
        token: &str,
        owner: &str,
        repo: &str,
        number: i64,
        event: &str,
        body: &str,
        login: &str,
        confirmed: bool,
    ) -> CoreResult<GitHubReview> {
        crate::github_actions::submit_review(
            token, owner, repo, number, event, body, login, confirmed,
        )
    }

    pub fn inbox(&self, filter: &str) -> CoreResult<Vec<InboxEvent>> {
        let workspace = self.require_workspace()?;
        self.db.with_conn(|conn| {
            crate::db::repositories::inbox::list(conn, &workspace.id, InboxFilter::parse(filter))
        })
    }

    pub fn activity_feed(&self, limit: i64) -> CoreResult<Vec<ActivityFeedItem>> {
        let workspace = self.require_workspace()?;
        self.db.with_conn(|conn| {
            crate::db::repositories::activity::list_workspace(conn, &workspace.id, limit)
        })
    }

    pub fn inbox_mark_read(&self, event_id: &str, read: bool) -> CoreResult<()> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|conn| {
            crate::db::repositories::inbox::mark_read(conn, &workspace.id, event_id, read)
        })
    }

    pub fn inbox_archive(&self, event_id: &str) -> CoreResult<()> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|conn| {
            crate::db::repositories::inbox::archive(conn, &workspace.id, event_id)
        })
    }

    pub fn inbox_mark_all_read(&self) -> CoreResult<i64> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|conn| {
            crate::db::repositories::inbox::mark_all_read(conn, &workspace.id)
        })
    }

    pub fn insights(&self, range_days: i64) -> CoreResult<InsightsSummary> {
        let workspace = self.require_workspace()?;
        self.db.with_conn(|conn| {
            crate::db::repositories::insights::summary(conn, &workspace.id, range_days)
        })
    }

    pub fn automation_rules(&self) -> CoreResult<Vec<AutomationRule>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| crate::db::repositories::automation::list_rules(conn, &workspace.id))
    }

    pub fn create_automation(&self, input: &AutomationInput) -> CoreResult<AutomationRule> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|conn| {
            crate::db::repositories::automation::create_rule(conn, &workspace.id, input)
        })
    }

    pub fn toggle_automation(&self, rule_id: &str, enabled: bool) -> CoreResult<AutomationRule> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|conn| {
            crate::db::repositories::automation::set_enabled(conn, &workspace.id, rule_id, enabled)
        })
    }

    pub fn delete_automation(&self, rule_id: &str) -> CoreResult<()> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|conn| {
            crate::db::repositories::automation::delete_rule(conn, &workspace.id, rule_id)
        })
    }

    pub fn preview_automation(&self, rule_id: &str) -> CoreResult<Vec<AutomationEvent>> {
        let workspace = self.require_workspace()?;
        self.db.with_conn(|conn| {
            crate::db::repositories::automation::preview(conn, &workspace.id, rule_id)
        })
    }

    pub fn run_automation(&self, rule_id: &str) -> CoreResult<i64> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|conn| {
            crate::db::repositories::automation::run_rule(conn, &workspace.id, rule_id)
        })
    }

    pub fn automation_runs(&self, rule_id: &str) -> CoreResult<Vec<AutomationRun>> {
        let workspace = self.require_workspace()?;
        self.db.with_conn(|conn| {
            crate::db::repositories::automation::runs(conn, &workspace.id, rule_id)
        })
    }

    pub fn dashboard(&self) -> CoreResult<Dashboard> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|conn| {
            crate::db::repositories::dashboard::get_or_create(conn, &workspace.id)
        })
    }

    pub fn dashboard_add_widget(&self, kind: &str, width: i64) -> CoreResult<Dashboard> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|conn| {
            crate::db::repositories::dashboard::add_widget(conn, &workspace.id, kind, width)
        })
    }

    pub fn dashboard_remove_widget(&self, widget_id: &str) -> CoreResult<Dashboard> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|conn| {
            crate::db::repositories::dashboard::remove_widget(conn, &workspace.id, widget_id)
        })
    }

    pub fn dashboard_update_width(&self, widget_id: &str, width: i64) -> CoreResult<Dashboard> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|conn| {
            crate::db::repositories::dashboard::update_width(conn, &workspace.id, widget_id, width)
        })
    }

    pub fn dashboard_reorder(&self, ids: &[String]) -> CoreResult<Dashboard> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|conn| {
            crate::db::repositories::dashboard::reorder(conn, &workspace.id, ids)
        })
    }

    pub fn git_available(&self) -> GitAvailability {
        crate::git::git_available()
    }

    pub fn list_repositories(&self) -> CoreResult<Vec<Repository>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| repo_registry::list(conn, &workspace.id))
    }

    pub fn add_local_repository(&self, input: RepositoryInput) -> CoreResult<Repository> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| repo_registry::add_local(tx, &workspace.id, &input))
    }

    pub fn remove_repository(&self, id: &str) -> CoreResult<()> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| repo_registry::remove(tx, &workspace.id, id))
    }

    pub fn get_repository(&self, id: &str) -> CoreResult<Repository> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| repo_registry::get(conn, &workspace.id, id))
    }

    pub fn list_project_repositories(&self, project_id: &str) -> CoreResult<Vec<Repository>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| repo_registry::list_for_project(conn, &workspace.id, project_id))
    }

    pub fn link_repository(&self, repository_id: &str, project_id: &str) -> CoreResult<Repository> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|tx| {
            repo_registry::link(tx, &workspace.id, repository_id, project_id)
        })
    }

    pub fn unlink_repository(
        &self,
        repository_id: &str,
        project_id: &str,
    ) -> CoreResult<Repository> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|tx| {
            repo_registry::unlink(tx, &workspace.id, repository_id, project_id)
        })
    }

    pub fn attach_repository_checkout(
        &self,
        repository_id: &str,
        path: &str,
    ) -> CoreResult<Repository> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|tx| {
            repo_registry::attach_checkout_checked(tx, &workspace.id, repository_id, path)?;
            repo_registry::get(tx, &workspace.id, repository_id)
        })
    }

    pub fn detach_repository_checkout(&self, checkout_id: &str) -> CoreResult<()> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| repo_registry::detach_checkout(tx, &workspace.id, checkout_id))
    }

    pub fn select_repository(&self, repository_id: Option<&str>) -> CoreResult<Option<Repository>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| repo_registry::set_selected(tx, &workspace.id, repository_id))
    }

    pub fn selected_repository(&self) -> CoreResult<Option<Repository>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| repo_registry::selected(conn, &workspace.id))
    }

    pub fn git_overview(&self, repository_id: &str) -> CoreResult<GitOverview> {
        let workspace = self.require_workspace()?;
        let repository = self
            .db
            .with_conn(|conn| repo_registry::get(conn, &workspace.id, repository_id))?;
        let checkout = repository
            .checkouts
            .first()
            .ok_or_else(|| CoreError::unsupported("This repository has no local checkout."))?;
        let path = std::path::Path::new(&checkout.path);
        let status = crate::git::status(path)?;
        let branches = crate::git::branches(path)?;
        let log = crate::git::log(path, 12)?;
        let diff = crate::git::diff_stat(path)?;
        self.db.with_conn(|conn| {
            repo_registry::touch(conn, &workspace.id, repository_id, &checkout.id)
        })?;
        Ok(GitOverview {
            repository,
            status,
            branches,
            log,
            diff,
        })
    }

    pub fn create_git_branch(&self, repository_id: &str, branch: &str) -> CoreResult<GitOverview> {
        let workspace = self.require_workspace()?;
        let repository = self
            .db
            .with_conn(|conn| repo_registry::get(conn, &workspace.id, repository_id))?;
        let path = repo_registry::checkout_path(&repository)?;
        crate::git::checkout_branch(&path, branch, true)?;
        self.git_overview(repository_id)
    }

    pub fn switch_git_branch(&self, repository_id: &str, branch: &str) -> CoreResult<GitOverview> {
        let workspace = self.require_workspace()?;
        let repository = self
            .db
            .with_conn(|conn| repo_registry::get(conn, &workspace.id, repository_id))?;
        let path = repo_registry::checkout_path(&repository)?;
        crate::git::checkout_branch(&path, branch, false)?;
        self.git_overview(repository_id)
    }

    pub fn git_log(&self, repository_id: &str, limit: usize) -> CoreResult<Vec<GitLogEntry>> {
        let workspace = self.require_workspace()?;
        let repository = self
            .db
            .with_conn(|conn| repo_registry::get(conn, &workspace.id, repository_id))?;
        crate::git::log(&repo_registry::checkout_path(&repository)?, limit)
    }

    pub fn git_diff(&self, repository_id: &str) -> CoreResult<GitDiffStat> {
        let workspace = self.require_workspace()?;
        let repository = self
            .db
            .with_conn(|conn| repo_registry::get(conn, &workspace.id, repository_id))?;
        crate::git::diff_stat(&repo_registry::checkout_path(&repository)?)
    }

    pub fn api_monitor(&self) -> CoreResult<ApiMonitorState> {
        let workspace = self.require_workspace()?;
        self.db.with_conn(|conn| {
            let budgets = api_budget::list_budgets(conn, &workspace.id)?;
            let requests = api_budget::list_requests(conn, &workspace.id)?;
            let queue = api_budget::list_queue(conn, &workspace.id)?;
            let control = api_budget::control(conn, &workspace.id)?;
            let account = api_budget::account_login(conn, &workspace.id)?;
            let last = api_budget::last_observation(conn, &workspace.id)?;
            Ok(ApiMonitorState {
                connected: account.is_some(),
                account_login: account,
                budgets,
                requests,
                queue,
                control,
                last_observation: last,
            })
        })
    }

    pub fn set_api_paused(&self, paused: bool, reason: &str) -> CoreResult<ApiControl> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| api_budget::set_paused(tx, &workspace.id, paused, reason))
    }

    pub fn export_api_diagnostics(&self) -> CoreResult<String> {
        let state = self.api_monitor()?;
        serde_json::to_string_pretty(&state)
            .map_err(|error| CoreError::internal(format!("Diagnostics encoding failed: {error}")))
    }

    pub fn list_project_cards(&self, query: ProjectQuery) -> CoreResult<Vec<Project>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| planning::list_projects(conn, &workspace.id, &query))
    }

    pub fn get_project(&self, id: &str) -> CoreResult<ProjectDetail> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| planning::project_detail(conn, &workspace.id, id))
    }

    pub fn create_project(&self, input: ProjectInput) -> CoreResult<Project> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| planning::create_project(tx, &workspace.id, &input))
    }

    pub fn update_project(&self, id: &str, input: ProjectInput) -> CoreResult<Project> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| planning::update_project(tx, &workspace.id, id, &input))
    }

    pub fn archive_project(&self, id: &str, archived: bool) -> CoreResult<Project> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| planning::archive_project(tx, &workspace.id, id, archived))
    }

    pub fn create_milestone(
        &self,
        project_id: &str,
        input: MilestoneInput,
    ) -> CoreResult<Milestone> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|tx| {
            planning::create_milestone(tx, &workspace.id, project_id, &input)
        })
    }

    pub fn update_milestone(
        &self,
        milestone_id: &str,
        input: MilestoneInput,
    ) -> CoreResult<Milestone> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|tx| {
            planning::update_milestone(tx, &workspace.id, milestone_id, &input)
        })
    }

    pub fn assign_milestone_issue(
        &self,
        milestone_id: &str,
        issue_id: &str,
        assigned: bool,
    ) -> CoreResult<()> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|tx| {
            planning::assign_milestone_issue(tx, &workspace.id, milestone_id, issue_id, assigned)
        })
    }

    pub fn list_initiatives(&self) -> CoreResult<Vec<Initiative>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| planning::list_initiatives(conn, &workspace.id))
    }

    pub fn get_initiative(&self, id: &str) -> CoreResult<InitiativeDetail> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| planning::initiative_detail(conn, &workspace.id, id))
    }

    pub fn create_initiative(&self, input: InitiativeInput) -> CoreResult<Initiative> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| planning::create_initiative(tx, &workspace.id, &input))
    }

    pub fn update_initiative(&self, id: &str, input: InitiativeInput) -> CoreResult<Initiative> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| planning::update_initiative(tx, &workspace.id, id, &input))
    }

    pub fn list_cycles(&self) -> CoreResult<Vec<Cycle>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| planning::list_cycles(conn, &workspace.id))
    }

    pub fn get_cycle(&self, id: &str) -> CoreResult<CycleDetail> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| planning::cycle_detail(conn, &workspace.id, id))
    }

    pub fn create_cycle(&self, input: CycleInput) -> CoreResult<Cycle> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| planning::create_cycle(tx, &workspace.id, &input))
    }

    pub fn update_cycle(&self, id: &str, input: CycleInput) -> CoreResult<Cycle> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| planning::update_cycle(tx, &workspace.id, id, &input))
    }

    pub fn assign_cycle_issue(
        &self,
        cycle_id: &str,
        issue_id: &str,
        assigned: bool,
    ) -> CoreResult<()> {
        let workspace = self.require_workspace()?;
        self.db.with_transaction(|tx| {
            planning::assign_cycle_issue(tx, &workspace.id, cycle_id, issue_id, assigned)
        })
    }

    pub fn list_documents(&self) -> CoreResult<Vec<Document>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| planning::list_documents(conn, &workspace.id))
    }

    pub fn get_document(&self, id: &str) -> CoreResult<Document> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| planning::get_document(conn, &workspace.id, id))
    }

    pub fn create_document(&self, input: DocumentInput) -> CoreResult<Document> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| planning::create_document(tx, &workspace.id, &input))
    }

    pub fn update_document(&self, id: &str, input: DocumentInput) -> CoreResult<Document> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| planning::update_document(tx, &workspace.id, id, &input))
    }

    pub fn delete_document(&self, id: &str) -> CoreResult<()> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| planning::delete_document(tx, &workspace.id, id))
    }

    pub fn export_document(&self, id: &str) -> CoreResult<String> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| planning::export_document(conn, &workspace.id, id))
    }

    pub fn list_saved_views(&self) -> CoreResult<Vec<SavedView>> {
        let workspace = self.require_workspace()?;
        self.db
            .with_conn(|conn| saved_views::list(conn, &workspace.id))
    }

    pub fn create_saved_view(&self, input: SavedViewInput) -> CoreResult<SavedView> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| saved_views::create(tx, &workspace.id, &input))
    }

    pub fn update_saved_view(&self, id: &str, input: SavedViewInput) -> CoreResult<SavedView> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| saved_views::update(tx, &workspace.id, id, &input))
    }

    pub fn delete_saved_view(&self, id: &str) -> CoreResult<()> {
        let workspace = self.require_workspace()?;
        self.db
            .with_transaction(|tx| saved_views::delete(tx, &workspace.id, id))
    }

    fn require_workspace(&self) -> CoreResult<Workspace> {
        self.db
            .workspace()?
            .ok_or_else(|| CoreError::not_found("Workspace"))
    }

    fn run_startup_maintenance(&mut self) {
        let settings = match self.db.settings() {
            Ok(settings) => settings,
            Err(error) => {
                self.startup_warning = Some(error.user_message());
                return;
            }
        };
        if !settings.auto_backup_enabled {
            return;
        }

        let needs_backup = match backup::latest_backup_age_hours(&self.backups_dir) {
            Ok(None) => true,
            Ok(Some(hours)) => hours >= backup::AUTO_BACKUP_INTERVAL_HOURS,
            Err(error) => {
                self.startup_warning = Some(error.user_message());
                true
            }
        };

        if needs_backup {
            match backup::create(&self.db, &self.backups_dir, &self.app_version) {
                Ok(_) => {
                    if let Err(error) = backup::prune(&self.backups_dir, settings.backup_retention)
                    {
                        self.startup_warning = Some(error.user_message());
                    }
                }
                Err(error) => {
                    // A failed automatic snapshot must not prevent local work.
                    self.startup_warning = Some(format!(
                        "Automatic backup could not be created: {}",
                        error.user_message()
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{LocalePreference, ThemePreference};

    fn service() -> (tempfile::TempDir, AppService) {
        let dir = tempfile::tempdir().expect("tempdir");
        let service = AppService::open(dir.path(), "0.1.0").expect("open service");
        (dir, service)
    }

    #[test]
    fn bootstraps_empty_installation() {
        let (_dir, service) = service();
        let bootstrap = service.bootstrap().expect("bootstrap");
        assert_eq!(
            bootstrap.schema_version,
            crate::db::migrations::CURRENT_SCHEMA_VERSION
        );
        assert!(bootstrap.workspace.is_none());
        assert!(!bootstrap.settings.onboarding_completed);
        assert!(Path::new(&bootstrap.db_path).is_file());
        // First launch with auto-backup enabled creates an initial snapshot.
        assert_eq!(bootstrap.backups.len(), 1);
    }

    #[test]
    fn creates_workspace_and_completes_onboarding() {
        let (_dir, service) = service();
        let workspace = service
            .create_workspace("Product Studio")
            .expect("workspace");
        assert_eq!(workspace.name, "Product Studio");

        let settings = service
            .update_settings(SettingsPatch {
                locale: Some(LocalePreference::Ru),
                theme: Some(ThemePreference::Light),
                ..Default::default()
            })
            .expect("settings");
        assert_eq!(settings.locale, LocalePreference::Ru);

        let completed = service.complete_onboarding().expect("complete");
        assert!(completed.onboarding_completed);

        let bootstrap = service.bootstrap().expect("bootstrap");
        assert_eq!(
            bootstrap.workspace.expect("workspace").name,
            "Product Studio"
        );
        assert!(bootstrap.settings.onboarding_completed);
    }

    #[test]
    fn restore_roundtrips_settings_and_workspace() {
        let (_dir, mut service) = service();
        service.create_workspace("Workspace A").expect("workspace");
        service
            .update_settings(SettingsPatch {
                theme: Some(ThemePreference::Dark),
                ..Default::default()
            })
            .expect("settings");
        let snapshot = service.create_backup().expect("backup");

        service
            .update_settings(SettingsPatch {
                theme: Some(ThemePreference::Light),
                ..Default::default()
            })
            .expect("settings");
        assert_eq!(
            service.bootstrap().expect("bootstrap").settings.theme,
            ThemePreference::Light
        );

        let result = service.restore_backup(&snapshot.id).expect("restore");
        assert_eq!(result.restored.id, snapshot.id);
        assert_eq!(result.bootstrap.settings.theme, ThemePreference::Dark);
        assert_eq!(
            result.bootstrap.workspace.expect("workspace").name,
            "Workspace A"
        );
        assert!(result.bootstrap.backups.iter().any(|b| b.id == snapshot.id));
        assert!(result
            .bootstrap
            .backups
            .iter()
            .any(|b| b.id == result.safety_backup.id));
    }

    #[test]
    fn failed_restore_leaves_service_usable() {
        let (_dir, mut service) = service();
        service.create_workspace("Workspace A").expect("workspace");
        let snapshot = service.create_backup().expect("backup");

        let path = Path::new(&snapshot.file_path).to_path_buf();
        let mut bytes = std::fs::read(&path).expect("read backup");
        bytes.truncate(bytes.len() / 2);
        std::fs::write(&path, bytes).expect("corrupt backup");

        let error = service.restore_backup(&snapshot.id).expect_err("must fail");
        assert_eq!(error.category(), crate::error::ErrorCategory::Backup);
        let bootstrap = service.bootstrap().expect("still usable");
        assert_eq!(bootstrap.workspace.expect("workspace").name, "Workspace A");
    }

    #[test]
    fn rejects_unknown_backup_id() {
        let (_dir, mut service) = service();
        assert!(service.restore_backup("does-not-exist").is_err());
        assert!(service.restore_backup("../forgedesk.db").is_err());
    }

    #[test]
    fn activity_feed_is_independent_of_inbox_archive_state() {
        let (_dir, service) = service();
        service
            .create_workspace("Activity workspace")
            .expect("workspace");
        let issue = service
            .create_issue(CreateIssueInput {
                title: "Feed event".to_owned(),
                description: String::new(),
                status_id: None,
                priority: crate::domain::Priority::default(),
                issue_type: crate::domain::IssueType::default(),
                estimate: None,
                start_date: None,
                due_date: None,
                project_id: None,
                parent_id: None,
                label_names: Vec::new(),
            })
            .expect("issue");

        let events = service.activity_feed(20).expect("activity feed");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].issue_key, issue.summary.key);
        assert_eq!(events[0].issue_title, "Feed event");
        assert_eq!(events[0].kind, "issue_created");

        service
            .inbox_archive(&events[0].id)
            .expect("archive inbox item");
        let events_after_archive = service.activity_feed(20).expect("activity feed");
        assert_eq!(events_after_archive.len(), 1);
        assert_eq!(events_after_archive[0].id, events[0].id);
    }
}
