use chrono::DateTime;
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

use crate::db::repositories::{api_budget, issues, planning, repo_registry, sync, workflow};
use crate::domain::api_budget::ApiObservationInput;
use crate::domain::issue::{CreateIssueInput, IssueType, Priority};
use crate::domain::planning::{PlanningStatus, ProjectInput, ProjectQuery};
use crate::domain::sync::RemoteIssueLink;
use crate::error::{CoreError, CoreResult};
use crate::github_client::{GitHubClient, GitHubIssue, GitHubRepo, RateSnapshot};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub imported: i64,
    pub skipped: i64,
    pub project_id: String,
    pub project_name: String,
    pub repository_id: String,
    pub has_more: bool,
    pub pages: u32,
    pub next_page: Option<u32>,
}

pub struct FetchedImport {
    pub repository: GitHubRepo,
    pub issues: Vec<GitHubIssue>,
    pub rates: Vec<RateSnapshot>,
    pub has_more: bool,
    pub pages: u32,
    pub start_page: u32,
    pub next_page: Option<u32>,
}

/// Network-only phase. No SQLite transaction or write is open here.
pub fn fetch_import(
    client: &GitHubClient,
    owner: &str,
    repo: &str,
    start_page: u32,
) -> CoreResult<FetchedImport> {
    let repository = client.get_repository(owner, repo)?;
    let mut issues = Vec::new();
    let mut rates = Vec::new();
    let start_page = start_page.max(1);
    let mut page = start_page;
    let mut pages = 0u32;
    let next_page = loop {
        let result = client.list_issues(owner, repo, page, 50)?;
        pages += 1;
        rates.push(result.rate);
        issues.extend(result.items);
        let more = result.next_page.is_some() && page < start_page + 4;
        if !more {
            break result.next_page;
        }
        page += 1;
    };
    Ok(FetchedImport {
        repository,
        issues,
        rates,
        has_more: next_page.is_some(),
        pages,
        start_page,
        next_page,
    })
}

/// Short SQLite apply phase. All remote data has already been fetched.
#[allow(clippy::too_many_arguments)]
pub fn apply_import(
    conn: &Connection,
    workspace_id: &str,
    account_login: &str,
    user_id: i64,
    fetched: &FetchedImport,
    target_project_id: Option<&str>,
) -> CoreResult<ImportResult> {
    let remote = &fetched.repository;
    let owner = remote.owner.login.clone();
    let repo_name = remote.name.clone();
    let host = "github.com";
    let canonical = repo_registry::normalize_remote(
        remote
            .clone_url
            .as_deref()
            .or(Some(remote.html_url.as_str()))
            .unwrap_or_default(),
    )
    .map(|value| value.canonical);
    let repository = repo_registry::upsert_remote(
        conn,
        workspace_id,
        &repo_registry::RemoteRepositoryInput {
            provider: "github".into(),
            host: host.into(),
            external_id: Some(remote.id.to_string()),
            owner: Some(owner.clone()),
            name: repo_name.clone(),
            full_name: remote.full_name.clone(),
            description: remote.description.clone(),
            visibility: remote
                .visibility
                .clone()
                .or_else(|| Some(if remote.private { "private" } else { "public" }.into())),
            default_branch: remote.default_branch.clone(),
            html_url: Some(remote.html_url.clone()),
            clone_url: remote.clone_url.clone(),
            canonical_remote_url: canonical,
            is_archived: remote.archived,
        },
    )?;

    let project = resolve_project(
        conn,
        workspace_id,
        &repository.id,
        target_project_id,
        &owner,
        &repo_name,
    )?;
    let statuses = workflow::statuses(conn, workspace_id)?;
    let default_status = statuses
        .iter()
        .find(|status| status.system_key.as_deref() == Some("todo"))
        .or_else(|| statuses.first())
        .ok_or_else(|| CoreError::internal("Workflow has no statuses."))?
        .id
        .clone();
    let completed_status = statuses
        .iter()
        .find(|status| status.system_key.as_deref() == Some("done"))
        .map(|status| status.id.clone());
    let account_id = format!("github:{user_id}");
    let mut imported = 0i64;
    let mut skipped = 0i64;

    for remote_issue in &fetched.issues {
        if remote_issue.pull_request.is_some() {
            continue;
        }
        let remote_id = remote_issue.id.to_string();
        if sync::find_by_remote(conn, workspace_id, "github", &account_id, &remote_id)?.is_some() {
            skipped += 1;
            continue;
        }
        let status_id = if remote_issue.state == "closed" {
            completed_status
                .clone()
                .unwrap_or_else(|| default_status.clone())
        } else {
            default_status.clone()
        };
        let label_names = remote_issue
            .labels
            .iter()
            .map(|label| label.name.clone())
            .collect::<Vec<_>>();
        let issue_id = issues::create(
            conn,
            workspace_id,
            &CreateIssueInput {
                title: remote_issue.title.clone(),
                description: remote_issue.body.clone().unwrap_or_default(),
                status_id: Some(status_id),
                priority: Priority::None,
                issue_type: IssueType::Task,
                estimate: None,
                start_date: None,
                due_date: None,
                project_id: Some(project.0.clone()),
                parent_id: None,
                label_names,
            },
        )?;
        conn.execute(
            "UPDATE issue SET source = 'github', remote_url = ?1 WHERE id = ?2",
            rusqlite::params![remote_issue.html_url, issue_id],
        )?;
        let link = RemoteIssueLink {
            id: uuid::Uuid::now_v7().to_string(),
            local_issue_id: issue_id,
            provider: "github".into(),
            account_id: account_id.clone(),
            remote_repository_key: Some(format!("{owner}/{repo_name}")),
            remote_id,
            remote_number: remote_issue.number,
            remote_url: remote_issue.html_url.clone(),
            remote_updated_at: remote_issue.updated_at.clone(),
            last_synced_at: Some(crate::domain::time::now_rfc3339()),
            field_ownership: "{\"title\":\"remote\",\"body\":\"remote\",\"status\":\"remote\"}"
                .into(),
            state: "linked".into(),
            created_at: String::new(),
            updated_at: String::new(),
        };
        sync::upsert_link(conn, workspace_id, &link)?;
        imported += 1;
    }
    for rate in &fetched.rates {
        record_rate(conn, workspace_id, rate)?;
    }
    let _ = account_login;
    Ok(ImportResult {
        imported,
        skipped,
        project_id: project.0,
        project_name: project.1,
        repository_id: repository.id,
        has_more: fetched.has_more,
        pages: fetched.pages,
        next_page: fetched.next_page,
    })
}

fn resolve_project(
    conn: &Connection,
    workspace_id: &str,
    repository_id: &str,
    target_project_id: Option<&str>,
    owner: &str,
    repo: &str,
) -> CoreResult<(String, String)> {
    let repository = repo_registry::get(conn, workspace_id, repository_id)?;
    let linked = repository
        .projects
        .iter()
        .filter(|project| is_active_project(conn, workspace_id, &project.id).unwrap_or(false))
        .cloned()
        .collect::<Vec<_>>();
    if let Some(target) = target_project_id {
        ensure_active_project(conn, workspace_id, target)?;
        if !linked.iter().any(|project| project.id == target) {
            repo_registry::link(conn, workspace_id, repository_id, target)?;
        }
        let name = conn.query_row(
            "SELECT name FROM project WHERE workspace_id = ?1 AND id = ?2",
            rusqlite::params![workspace_id, target],
            |row| row.get::<_, String>(0),
        )?;
        return Ok((target.to_owned(), name));
    }
    match linked.as_slice() {
        [project] => return Ok((project.id.clone(), project.name.clone())),
        projects if projects.len() > 1 => {
            return Err(CoreError::conflict(
                "This repository is linked to multiple projects; choose a target project.",
            ));
        }
        _ => {}
    }
    let name = format!("{owner}/{repo}");
    let existing = planning::list_projects(
        conn,
        workspace_id,
        &ProjectQuery {
            search: Some(name.clone()),
            include_archived: false,
            ..Default::default()
        },
    )?
    .into_iter()
    .find(|project| project.name.eq_ignore_ascii_case(&name));
    let project = match existing {
        Some(project) => project,
        None => planning::create_project(
            conn,
            workspace_id,
            &ProjectInput {
                name: name.clone(),
                description: format!("Imported from GitHub repository {name}."),
                status: Some(PlanningStatus::Active),
                priority: Some(Priority::None),
                ..Default::default()
            },
        )?,
    };
    repo_registry::link(conn, workspace_id, repository_id, &project.id)?;
    Ok((project.id, project.name))
}

fn is_active_project(conn: &Connection, workspace_id: &str, project_id: &str) -> CoreResult<bool> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM project WHERE workspace_id = ?1 AND id = ?2 AND archived_at IS NULL",
            rusqlite::params![workspace_id, project_id],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false))
}

fn ensure_active_project(
    conn: &Connection,
    workspace_id: &str,
    project_id: &str,
) -> CoreResult<()> {
    if !is_active_project(conn, workspace_id, project_id)? {
        return Err(CoreError::validation(
            "project",
            "Target project is archived or does not belong to this workspace.",
        ));
    }
    Ok(())
}

fn record_rate(conn: &Connection, workspace_id: &str, rate: &RateSnapshot) -> CoreResult<()> {
    let reset = rate.reset.and_then(|value| {
        DateTime::<chrono::Utc>::from_timestamp(value, 0).map(|date| date.to_rfc3339())
    });
    let resource = rate.resource.clone().unwrap_or_else(|| "core".into());
    let input = ApiObservationInput {
        api_family: "github_rest".into(),
        resource_name: resource,
        limit_value: rate.limit,
        remaining_value: rate.remaining,
        used_value: rate.used,
        reset_at_utc: reset,
        http_status: Some(rate.http_status as i64),
        source: "response_headers".into(),
        is_estimate: false,
        secondary_pause_until: None,
        notes: "GitHub issue import".into(),
    };
    if input.validate().is_ok() {
        api_budget::record_observation(conn, workspace_id, &input)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_result_serializes() {
        let result = ImportResult {
            imported: 2,
            skipped: 1,
            project_id: "p".into(),
            project_name: "o/r".into(),
            repository_id: "r".into(),
            has_more: false,
            pages: 1,
            next_page: None,
        };
        let json = serde_json::to_value(result).expect("json");
        assert_eq!(json["imported"], 2);
        assert_eq!(json["projectName"], "o/r");
        assert_eq!(json["repositoryId"], "r");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::db::Database;
    use crate::github_client::{GitHubIssue, GitHubOwner, RateSnapshot};

    fn repository() -> GitHubRepo {
        GitHubRepo {
            id: 123,
            name: "repo".into(),
            full_name: "owner/repo".into(),
            private: false,
            default_branch: Some("main".into()),
            html_url: "https://github.com/owner/repo".into(),
            description: Some("test".into()),
            updated_at: None,
            owner: GitHubOwner {
                login: "owner".into(),
            },
            permissions: None,
            clone_url: Some("https://github.com/owner/repo.git".into()),
            visibility: Some("public".into()),
            archived: false,
        }
    }

    fn issue(id: i64, number: i64) -> GitHubIssue {
        GitHubIssue {
            id,
            number,
            title: format!("Issue {number}"),
            body: Some("body".into()),
            state: "open".into(),
            labels: vec![],
            html_url: format!("https://github.com/owner/repo/issues/{number}"),
            updated_at: None,
            user: None,
            pull_request: None,
        }
    }

    fn fetched(issues: Vec<GitHubIssue>, has_more: bool) -> FetchedImport {
        FetchedImport {
            repository: repository(),
            issues,
            rates: vec![RateSnapshot {
                limit: Some(5000),
                remaining: Some(4999),
                used: Some(1),
                reset: None,
                resource: Some("core".into()),
                http_status: 200,
            }],
            has_more,
            pages: 1,
            start_page: 1,
            next_page: has_more.then_some(6),
        }
    }

    fn db() -> (tempfile::TempDir, Database, String) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("db");
        let workspace = db
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "Import"))
            .expect("workspace");
        (dir, db, workspace.id)
    }

    #[test]
    fn import_creates_repository_project_link_and_is_idempotent_after_reopen() {
        let (dir, db, workspace_id) = db();
        let data = fetched(vec![issue(1, 1), issue(2, 2)], false);
        let first = db
            .with_transaction(|tx| apply_import(tx, &workspace_id, "login", 7, &data, None))
            .expect("import");
        assert_eq!(first.imported, 2);
        assert!(!first.has_more);
        let repository_id = first.repository_id.clone();
        let project_id = first.project_id.clone();
        db.with_conn(|conn| {
            let repository = repo_registry::get(conn, &workspace_id, &repository_id)?;
            assert_eq!(repository.projects.len(), 1);
            let count: i64 = conn.query_row(
                "SELECT count(*) FROM issue WHERE workspace_id = ?1 AND project_id = ?2",
                rusqlite::params![workspace_id, project_id],
                |row| row.get(0),
            )?;
            assert_eq!(count, 2);
            let link = sync::find_by_remote(conn, &workspace_id, "github", "github:7", "1")?
                .expect("link");
            assert_eq!(link.remote_repository_key.as_deref(), Some("owner/repo"));
            Ok(())
        })
        .expect("relationships");
        drop(db);

        let reopened = Database::open(dir.path().join("test.db")).expect("reopen");
        reopened
            .with_conn(|conn| {
                let repository = repo_registry::get(conn, &workspace_id, &repository_id)?;
                assert_eq!(repository.projects.len(), 1);
                Ok(())
            })
            .expect("persisted");
        let second = reopened
            .with_transaction(|tx| apply_import(tx, &workspace_id, "login", 7, &data, None))
            .expect("second import");
        assert_eq!(second.imported, 0);
        assert_eq!(second.skipped, 2);
    }

    #[test]
    fn import_requires_explicit_target_when_multiple_projects_are_linked() {
        let (_dir, db, workspace_id) = db();
        let data = fetched(vec![issue(1, 1)], false);
        let first = db
            .with_transaction(|tx| apply_import(tx, &workspace_id, "login", 7, &data, None))
            .expect("import");
        let second_project = db
            .with_conn(|conn| {
                crate::db::repositories::planning::create_project(
                    conn,
                    &workspace_id,
                    &ProjectInput {
                        name: "Second".into(),
                        status: Some(PlanningStatus::Active),
                        ..Default::default()
                    },
                )
            })
            .expect("project");
        db.with_transaction(|tx| {
            repo_registry::link(tx, &workspace_id, &first.repository_id, &second_project.id)
        })
        .expect("link");
        assert!(db
            .with_transaction(|tx| { apply_import(tx, &workspace_id, "login", 7, &data, None) })
            .is_err());
        let targeted = db
            .with_transaction(|tx| {
                apply_import(
                    tx,
                    &workspace_id,
                    "login",
                    7,
                    &data,
                    Some(&second_project.id),
                )
            })
            .expect("targeted");
        assert_eq!(targeted.project_id, second_project.id);
        assert_eq!(targeted.imported, 0, "existing remote issue stays linked");
        assert_eq!(targeted.skipped, 1);
    }

    #[test]
    fn import_rejects_archived_target_and_reports_partial() {
        let (_dir, db, workspace_id) = db();
        let data = fetched(vec![issue(1, 1)], true);
        let first = db
            .with_transaction(|tx| apply_import(tx, &workspace_id, "login", 7, &data, None))
            .expect("import");
        assert!(first.has_more);
        db.with_conn(|conn| {
            conn.execute(
                "UPDATE project SET archived_at = ?1 WHERE id = ?2",
                rusqlite::params![crate::domain::time::now_rfc3339(), first.project_id],
            )?;
            Ok(())
        })
        .expect("archive");
        assert!(db
            .with_transaction(|tx| {
                apply_import(
                    tx,
                    &workspace_id,
                    "login",
                    7,
                    &data,
                    Some(&first.project_id),
                )
            })
            .is_err());
    }
}
