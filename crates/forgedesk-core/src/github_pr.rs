use serde::Serialize;

use crate::error::CoreResult;
use crate::github_client::{GitHubCheckRun, GitHubClient, GitHubCommit, GitHubPull, GitHubReview};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestDetail {
    pub pull: GitHubPull,
    pub commits: Vec<GitHubCommit>,
    pub reviews: Vec<GitHubReview>,
    pub checks: Vec<GitHubCheckRun>,
}

pub fn list_pulls(
    token: &str,
    owner: &str,
    repo: &str,
    state: &str,
) -> CoreResult<(Vec<GitHubPull>, bool)> {
    let client = GitHubClient::new(token)?;
    let page = client.list_pulls(owner, repo, state)?;
    Ok((page.items, page.next_page.is_some()))
}

pub fn pull_detail(
    token: &str,
    owner: &str,
    repo: &str,
    number: i64,
) -> CoreResult<PullRequestDetail> {
    let client = GitHubClient::new(token)?;
    let pull = client.get_pull(owner, repo, number)?;
    let commits = client.list_pull_commits(owner, repo, number)?.items;
    let reviews = client.list_pull_reviews(owner, repo, number)?.items;
    let checks = client.list_check_runs(owner, repo, &pull.head.sha)?;
    Ok(PullRequestDetail {
        pull,
        commits,
        reviews,
        checks,
    })
}

#[cfg(test)]
mod live_tests {
    use super::*;

    #[test]
    #[ignore = "requires GITHUB_TOKEN and external network; read-only public repo"]
    fn live_lists_and_loads_pull_detail() {
        let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN");
        let (pulls, _) = list_pulls(&token, "cli", "cli", "open").expect("list");
        assert!(!pulls.is_empty(), "expected open pull requests");
        let detail = pull_detail(&token, "cli", "cli", pulls[0].number).expect("detail");
        assert_eq!(detail.pull.number, pulls[0].number);
        assert!(!detail.pull.head.sha.is_empty());
        // Checks may be empty for a specific PR, but the endpoint must not error.
        let _ = detail.checks.len();
    }
}
