use serde::Serialize;

use crate::error::CoreResult;
use crate::github_client::{GitHubCheckRun, GitHubClient, GitHubCommit, GitHubSearchIssue};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CiOverview {
    pub default_branch: String,
    pub commits: Vec<GitHubCommit>,
    pub checks: Vec<GitHubCheckRun>,
}

pub fn review_requested(
    token: &str,
    owner: &str,
    repo: &str,
    login: &str,
) -> CoreResult<Vec<GitHubSearchIssue>> {
    GitHubClient::new(token)?.review_requested(owner, repo, login)
}

pub fn ci_overview(token: &str, owner: &str, repo: &str) -> CoreResult<CiOverview> {
    let client = GitHubClient::new(token)?;
    let repository = client.get_repository(owner, repo)?;
    let branch = repository.default_branch.unwrap_or_else(|| "main".into());
    let commits = client.list_commits(owner, repo, 20)?;
    let checks = match commits.first() {
        Some(commit) => client.list_check_runs(owner, repo, &commit.sha)?,
        None => Vec::new(),
    };
    Ok(CiOverview {
        default_branch: branch,
        commits,
        checks,
    })
}

pub fn commit_checks(
    token: &str,
    owner: &str,
    repo: &str,
    reference: &str,
) -> CoreResult<Vec<GitHubCheckRun>> {
    GitHubClient::new(token)?.list_check_runs(owner, repo, reference)
}

#[cfg(test)]
mod live_tests {
    use super::*;

    #[test]
    #[ignore = "requires GITHUB_TOKEN and external network; read-only public repo"]
    fn live_ci_overview_and_review_search() {
        let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN");
        let overview = ci_overview(&token, "cli", "cli").expect("ci overview");
        assert!(!overview.default_branch.is_empty());
        assert!(!overview.commits.is_empty());
        let _ = review_requested(&token, "cli", "cli", "mindwork64").expect("review search");
    }
}
