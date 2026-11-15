use chrono::DateTime;
use rusqlite::Connection;
use serde::Serialize;

use crate::db::repositories::{api_budget, issues, planning, sync, workflow};
use crate::domain::api_budget::ApiObservationInput;
use crate::domain::issue::{CreateIssueInput, IssueType, Priority};
use crate::domain::planning::{PlanningStatus, ProjectInput, ProjectQuery};
use crate::domain::sync::RemoteIssueLink;
use crate::error::{CoreError, CoreResult};
use crate::github_client::{GitHubClient, RateSnapshot};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub imported: i64,
    pub skipped: i64,
    pub project_id: String,
    pub project_name: String,
    pub has_more: bool,
    pub pages: u32,
}

pub fn import_issues(
    conn: &Connection,
    workspace_id: &str,
    token: &str,
    account_login: &str,
    user_id: i64,
    owner: &str,
    repo: &str,
) -> CoreResult<ImportResult> {
    let client = GitHubClient::new(token)?;
    let project = ensure_project(conn, workspace_id, owner, repo)?;
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
    let mut page = 1u32;
    let mut pages = 0u32;

    let has_more = loop {
        let result = client.list_issues(owner, repo, page, 50)?;
        pages += 1;
        record_rate(conn, workspace_id, &result.rate)?;
        for remote in result.items {
            if remote.pull_request.is_some() {
                continue;
            }
            let remote_id = remote.id.to_string();
            if sync::find_by_remote(conn, workspace_id, "github", &account_id, &remote_id)?
                .is_some()
            {
                skipped += 1;
                continue;
            }
            let status_id = if remote.state == "closed" {
                completed_status
                    .clone()
                    .unwrap_or_else(|| default_status.clone())
            } else {
                default_status.clone()
            };
            let label_names = remote
                .labels
                .iter()
                .map(|label| label.name.clone())
                .collect::<Vec<_>>();
            let issue_id = issues::create(
                conn,
                workspace_id,
                &CreateIssueInput {
                    title: remote.title,
                    description: remote.body.unwrap_or_default(),
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
                rusqlite::params![remote.html_url, issue_id],
            )?;
            let link = RemoteIssueLink {
                id: uuid::Uuid::now_v7().to_string(),
                local_issue_id: issue_id,
                provider: "github".into(),
                account_id: account_id.clone(),
                repository_id: Some(format!("{owner}/{repo}")),
                remote_id,
                remote_number: remote.number,
                remote_url: remote.html_url,
                remote_updated_at: remote.updated_at,
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
        let more = result.next_page.is_some() && page < 5;
        if !more {
            break more;
        }
        page += 1;
    };

    // Account identity is intentionally not written to SQLite; the link account id
    // carries the numeric GitHub user id only.
    let _ = account_login;
    Ok(ImportResult {
        imported,
        skipped,
        project_id: project.0,
        project_name: project.1,
        has_more,
        pages,
    })
}

fn ensure_project(
    conn: &Connection,
    workspace_id: &str,
    owner: &str,
    repo: &str,
) -> CoreResult<(String, String)> {
    let name = format!("{owner}/{repo}");
    let existing = planning::list_projects(
        conn,
        workspace_id,
        &ProjectQuery {
            search: Some(name.clone()),
            include_archived: true,
            ..Default::default()
        },
    )?
    .into_iter()
    .find(|project| project.name.eq_ignore_ascii_case(&name));
    if let Some(project) = existing {
        return Ok((project.id, project.name));
    }
    let project = planning::create_project(
        conn,
        workspace_id,
        &ProjectInput {
            name: name.clone(),
            description: format!("Imported from GitHub repository {name}."),
            status: Some(PlanningStatus::Active),
            priority: Some(Priority::None),
            ..Default::default()
        },
    )?;
    Ok((project.id, project.name))
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
            has_more: false,
            pages: 1,
        };
        let json = serde_json::to_value(result).expect("json");
        assert_eq!(json["imported"], 2);
        assert_eq!(json["projectName"], "o/r");
    }
}

#[cfg(test)]
mod live_tests {
    use super::*;
    use crate::db::Database;

    #[test]
    #[ignore = "requires GITHUB_TOKEN and external network; read-only public repo import"]
    fn live_imports_public_repo_and_is_idempotent() {
        let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN");
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("live.db")).expect("db");
        let workspace = db
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "Live"))
            .expect("workspace");
        let first = db
            .with_conn(|conn| {
                import_issues(conn, &workspace.id, &token, "live-test", 0, "cli", "cli")
            })
            .expect("first import");
        assert!(first.imported > 0, "expected imported issues");
        assert!(first.pages >= 1);
        let second = db
            .with_conn(|conn| {
                import_issues(conn, &workspace.id, &token, "live-test", 0, "cli", "cli")
            })
            .expect("second import");
        assert_eq!(second.imported, 0, "second import must not duplicate");
        assert!(
            second.skipped >= first.imported,
            "existing links must be skipped"
        );
    }
}
