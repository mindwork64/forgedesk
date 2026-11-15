use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{CoreError, CoreResult};

const DEFAULT_API: &str = "https://api.github.com";

#[derive(Debug, Clone, Default)]
pub struct RateSnapshot {
    pub limit: Option<i64>,
    pub remaining: Option<i64>,
    pub used: Option<i64>,
    pub reset: Option<i64>,
    pub resource: Option<String>,
    pub http_status: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub default_branch: Option<String>,
    pub html_url: String,
    pub description: Option<String>,
    pub updated_at: Option<String>,
    pub owner: GitHubOwner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubOwner {
    pub login: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubLabel {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubIssue {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    #[serde(default)]
    pub labels: Vec<GitHubLabel>,
    pub html_url: String,
    pub updated_at: Option<String>,
    pub user: Option<GitHubOwner>,
    pub pull_request: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_page: Option<u32>,
    pub rate: RateSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRef {
    #[serde(rename = "ref")]
    pub reference: String,
    pub sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPull {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub state: String,
    #[serde(default)]
    pub draft: bool,
    pub user: Option<GitHubOwner>,
    pub head: GitHubRef,
    pub base: GitHubRef,
    pub updated_at: String,
    pub created_at: String,
    pub html_url: String,
    pub body: Option<String>,
    pub mergeable: Option<bool>,
    pub merged_at: Option<String>,
    pub additions: Option<i64>,
    pub deletions: Option<i64>,
    pub changed_files: Option<i64>,
    pub commits: Option<i64>,
    #[serde(default)]
    pub requested_reviewers: Vec<GitHubOwner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCommit {
    pub sha: String,
    pub commit: GitHubCommitData,
    pub author: Option<GitHubOwner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCommitData {
    pub message: String,
    pub author: Option<GitHubCommitAuthor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCommitAuthor {
    pub name: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubReview {
    pub id: i64,
    pub user: Option<GitHubOwner>,
    pub state: String,
    pub body: Option<String>,
    pub submitted_at: Option<String>,
    pub html_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCheckRun {
    pub id: i64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub html_url: Option<String>,
    pub details_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubFile {
    pub filename: String,
    pub status: String,
    pub additions: i64,
    pub deletions: i64,
    pub changes: i64,
    pub patch: Option<String>,
    pub previous_filename: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CheckRunsResponse {
    #[serde(default)]
    check_runs: Vec<GitHubCheckRun>,
}

pub struct GitHubClient {
    http: Client,
    token: String,
    base: String,
}

impl GitHubClient {
    pub fn new(token: impl Into<String>) -> CoreResult<Self> {
        let http = Client::builder()
            .user_agent("ForgeDesk/0.1 (+https://github.com/mindwork64/forgedesk)")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|error| CoreError::network(format!("HTTP client failed: {error}")))?;
        let base = std::env::var("FORGEDESK_GITHUB_API_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_API.to_owned());
        Ok(Self {
            http,
            token: token.into(),
            base: base.trim_end_matches('/').to_owned(),
        })
    }

    pub fn list_repositories(&self) -> CoreResult<Page<GitHubRepo>> {
        self.get_page("/user/repos?per_page=100&sort=updated&affiliation=owner,collaborator,organization_member")
    }

    pub fn list_issues(
        &self,
        owner: &str,
        repo: &str,
        page: u32,
        per_page: u32,
    ) -> CoreResult<Page<GitHubIssue>> {
        let owner = encode(owner);
        let repo = encode(repo);
        self.get_page(&format!(
            "/repos/{owner}/{repo}/issues?state=all&per_page={}&page={}",
            per_page.clamp(1, 100),
            page.max(1)
        ))
    }

    pub fn list_pulls(&self, owner: &str, repo: &str, state: &str) -> CoreResult<Page<GitHubPull>> {
        let owner = encode(owner);
        let repo = encode(repo);
        let state = match state {
            "closed" | "all" => state,
            _ => "open",
        };
        self.get_page(&format!(
            "/repos/{owner}/{repo}/pulls?state={state}&per_page=50&sort=updated&direction=desc"
        ))
    }

    pub fn get_pull(&self, owner: &str, repo: &str, number: i64) -> CoreResult<GitHubPull> {
        let path = format!("/repos/{}/{}/pulls/{}", encode(owner), encode(repo), number);
        self.get_json(&path)
    }

    pub fn list_pull_commits(
        &self,
        owner: &str,
        repo: &str,
        number: i64,
    ) -> CoreResult<Page<GitHubCommit>> {
        let path = format!(
            "/repos/{}/{}/pulls/{}/commits?per_page=50",
            encode(owner),
            encode(repo),
            number
        );
        self.get_page(&path)
    }

    pub fn list_pull_reviews(
        &self,
        owner: &str,
        repo: &str,
        number: i64,
    ) -> CoreResult<Page<GitHubReview>> {
        let path = format!(
            "/repos/{}/{}/pulls/{}/reviews?per_page=50",
            encode(owner),
            encode(repo),
            number
        );
        self.get_page(&path)
    }

    pub fn list_pull_files(
        &self,
        owner: &str,
        repo: &str,
        number: i64,
    ) -> CoreResult<Vec<GitHubFile>> {
        let path = format!(
            "/repos/{}/{}/pulls/{}/files?per_page=100",
            encode(owner),
            encode(repo),
            number
        );
        let page: Page<GitHubFile> = self.get_page(&path)?;
        Ok(page.items)
    }

    pub fn list_check_runs(
        &self,
        owner: &str,
        repo: &str,
        reference: &str,
    ) -> CoreResult<Vec<GitHubCheckRun>> {
        let path = format!(
            "/repos/{}/{}/commits/{}/check-runs?per_page=50",
            encode(owner),
            encode(repo),
            encode(reference)
        );
        let response: CheckRunsResponse = self.get_json(&path)?;
        Ok(response.check_runs)
    }

    fn get_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> CoreResult<T> {
        let url = format!("{}{}", self.base, path);
        let response = self
            .http
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .bearer_auth(&self.token)
            .send()
            .map_err(|error| CoreError::network(format!("GitHub request failed: {error}")))?;
        let status = response.status();
        let rate = rate_from_headers(&response);
        let text = response
            .text()
            .map_err(|error| CoreError::network(format!("GitHub response failed: {error}")))?;
        if !status.is_success() {
            return Err(map_status(status, &text, rate.remaining));
        }
        serde_json::from_str(&text)
            .map_err(|error| CoreError::network(format!("GitHub JSON parsing failed: {error}")))
    }

    fn get_page<T: for<'de> Deserialize<'de>>(&self, path: &str) -> CoreResult<Page<T>> {
        let url = format!("{}{}", self.base, path);
        let response = self
            .http
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .bearer_auth(&self.token)
            .send()
            .map_err(|error| CoreError::network(format!("GitHub request failed: {error}")))?;
        let status = response.status();
        let rate = rate_from_headers(&response);
        let link = response
            .headers()
            .get("link")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let text = response
            .text()
            .map_err(|error| CoreError::network(format!("GitHub response failed: {error}")))?;
        if !status.is_success() {
            return Err(map_status(status, &text, rate.remaining));
        }
        let items: Vec<T> = serde_json::from_str(&text)
            .map_err(|error| CoreError::network(format!("GitHub JSON parsing failed: {error}")))?;
        Ok(Page {
            items,
            next_page: link.as_deref().and_then(parse_next_page),
            rate,
        })
    }
}

fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                (byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

fn rate_from_headers(response: &reqwest::blocking::Response) -> RateSnapshot {
    let header = |name: &str| {
        response
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<i64>().ok())
    };
    RateSnapshot {
        limit: header("x-ratelimit-limit"),
        remaining: header("x-ratelimit-remaining"),
        used: header("x-ratelimit-used"),
        reset: header("x-ratelimit-reset"),
        resource: response
            .headers()
            .get("x-ratelimit-resource")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned),
        http_status: response.status().as_u16(),
    }
}

fn parse_next_page(link: &str) -> Option<u32> {
    for part in link.split(',') {
        if !part.contains("rel=\"next\"") {
            continue;
        }
        let raw = part.split("page=").nth(1)?;
        let digits: String = raw.chars().take_while(|ch| ch.is_ascii_digit()).collect();
        return digits.parse().ok();
    }
    None
}

fn map_status(status: StatusCode, body: &str, remaining: Option<i64>) -> CoreError {
    let message = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("message")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| format!("HTTP {}", status.as_u16()));
    if status == StatusCode::UNAUTHORIZED {
        return CoreError::auth(format!("GitHub rejected the token: {message}"));
    }
    if status == StatusCode::FORBIDDEN && remaining == Some(0) {
        return CoreError::auth(format!("GitHub rate limit exhausted: {message}"));
    }
    if status == StatusCode::FORBIDDEN {
        return CoreError::auth(format!("GitHub permission denied: {message}"));
    }
    CoreError::network(format!("GitHub request failed: {message}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_next_page_and_encodes_names() {
        assert_eq!(
            parse_next_page(
                "<https://api.github.com/user/repos?page=2>; rel=\"next\", <...>; rel=\"last\""
            ),
            Some(2)
        );
        assert_eq!(parse_next_page("<...>; rel=\"last\""), None);
        assert_eq!(encode("owner/repo name"), "owner%2Frepo%20name");
    }

    #[test]
    fn maps_rate_limit_403_to_auth_error() {
        let error = map_status(
            StatusCode::FORBIDDEN,
            r#"{"message":"API rate limit exceeded"}"#,
            Some(0),
        );
        assert_eq!(error.category(), crate::error::ErrorCategory::Auth);
    }
}
