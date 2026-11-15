use serde::Serialize;

use crate::error::{CoreError, CoreResult};
use crate::github_client::{
    GitHubClient, GitHubMergeResult, GitHubPermissions, GitHubPull, GitHubReview,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullCapabilities {
    pub can_manage: bool,
    pub can_review: bool,
}

fn permissions(owner: &str, repo: &str, token: &str) -> CoreResult<GitHubPermissions> {
    Ok(GitHubClient::new(token)?
        .get_repository(owner, repo)?
        .permissions
        .unwrap_or_default())
}

pub fn capabilities(token: &str, owner: &str, repo: &str) -> CoreResult<PullCapabilities> {
    let permissions = permissions(owner, repo, token)?;
    Ok(PullCapabilities {
        can_manage: permissions.can_manage(),
        can_review: permissions.can_review() || permissions.pull,
    })
}

pub fn merge_pull(
    token: &str,
    owner: &str,
    repo: &str,
    number: i64,
    method: &str,
    expected_sha: Option<&str>,
    confirmed: bool,
) -> CoreResult<GitHubMergeResult> {
    if !confirmed {
        return Err(CoreError::validation(
            "confirmed",
            "Merge must be explicitly confirmed.",
        ));
    }
    if !matches!(method, "merge" | "squash" | "rebase") {
        return Err(CoreError::validation("method", "Unsupported merge method."));
    }
    if !permissions(owner, repo, token)?.can_manage() {
        return Err(CoreError::permission(
            "You do not have permission to merge this pull request.",
        ));
    }
    GitHubClient::new(token)?.merge_pull(owner, repo, number, method, expected_sha)
}

pub fn close_pull(
    token: &str,
    owner: &str,
    repo: &str,
    number: i64,
    confirmed: bool,
) -> CoreResult<GitHubPull> {
    if !confirmed {
        return Err(CoreError::validation(
            "confirmed",
            "Closing a pull request must be explicitly confirmed.",
        ));
    }
    if !permissions(owner, repo, token)?.can_manage() {
        return Err(CoreError::permission(
            "You do not have permission to close this pull request.",
        ));
    }
    GitHubClient::new(token)?.close_pull(owner, repo, number)
}

#[allow(clippy::too_many_arguments)]
pub fn submit_review(
    token: &str,
    owner: &str,
    repo: &str,
    number: i64,
    event: &str,
    body: &str,
    login: &str,
    confirmed: bool,
) -> CoreResult<GitHubReview> {
    if !confirmed {
        return Err(CoreError::validation(
            "confirmed",
            "Review submission must be explicitly confirmed.",
        ));
    }
    if !matches!(event, "APPROVE" | "REQUEST_CHANGES" | "COMMENT") {
        return Err(CoreError::validation("event", "Unsupported review event."));
    }
    if event == "REQUEST_CHANGES" && body.trim().is_empty() {
        return Err(CoreError::validation(
            "body",
            "Requesting changes requires a comment.",
        ));
    }
    if !permissions(owner, repo, token)?.can_review() {
        return Err(CoreError::permission(
            "You do not have permission to review this pull request.",
        ));
    }
    let client = GitHubClient::new(token)?;
    let pull = client.get_pull(owner, repo, number)?;
    if event == "APPROVE" && pull.user.as_ref().map(|user| user.login.as_str()) == Some(login) {
        return Err(CoreError::validation(
            "event",
            "You cannot approve your own pull request.",
        ));
    }
    client.submit_review(owner, repo, number, event, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_requires_explicit_confirmation() {
        let error = merge_pull("token", "o", "r", 1, "merge", None, false).expect_err("confirm");
        assert_eq!(error.category(), crate::error::ErrorCategory::Validation);
    }

    #[test]
    fn merge_rejects_unknown_method() {
        let error =
            merge_pull("token", "o", "r", 1, "fast-forward", None, true).expect_err("method");
        assert_eq!(error.category(), crate::error::ErrorCategory::Validation);
    }

    #[test]
    fn close_requires_explicit_confirmation() {
        let error = close_pull("token", "o", "r", 1, false).expect_err("confirm");
        assert_eq!(error.category(), crate::error::ErrorCategory::Validation);
    }

    #[test]
    fn request_changes_requires_body() {
        let error = submit_review("token", "o", "r", 1, "REQUEST_CHANGES", " ", "user", true)
            .expect_err("body");
        assert_eq!(error.category(), crate::error::ErrorCategory::Validation);
    }

    #[test]
    fn review_rejects_unknown_event() {
        let error =
            submit_review("token", "o", "r", 1, "LIKE", "", "user", true).expect_err("event");
        assert_eq!(error.category(), crate::error::ErrorCategory::Validation);
    }
}
