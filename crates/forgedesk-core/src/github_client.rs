use std::collections::HashSet;
use std::time::{Duration, Instant};

use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{CoreError, CoreResult};
use crate::github_dispatcher::{self, RequestClass};

const DEFAULT_API: &str = "https://api.github.com";

#[derive(Debug, Clone, Default, Serialize)]
pub struct RateSnapshot {
    pub limit: Option<i64>,
    pub remaining: Option<i64>,
    pub used: Option<i64>,
    pub reset: Option<i64>,
    pub resource: Option<String>,
    pub http_status: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
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
    #[serde(default)]
    pub permissions: Option<GitHubPermissions>,
    #[serde(default)]
    pub clone_url: Option<String>,
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(default)]
    pub archived: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitHubPermissions {
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub maintain: bool,
    #[serde(default)]
    pub push: bool,
    #[serde(default)]
    pub triage: bool,
    #[serde(default)]
    pub pull: bool,
}

impl GitHubPermissions {
    pub fn can_manage(&self) -> bool {
        self.admin || self.maintain || self.push
    }

    pub fn can_review(&self) -> bool {
        self.can_manage() || self.triage || self.pull
    }
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

#[derive(Debug, Clone, Copy, Default)]
pub struct GitHubIssueUpdate<'a> {
    pub title: Option<&'a str>,
    pub body: Option<&'a str>,
    pub state: Option<&'a str>,
    pub labels: Option<&'a [String]>,
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
pub struct GitHubSearchIssue {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub state: String,
    pub html_url: String,
    pub updated_at: String,
    pub user: Option<GitHubOwner>,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubSearchResult {
    #[serde(default)]
    items: Vec<GitHubSearchIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubReviewComment {
    pub id: i64,
    pub body: String,
    pub path: String,
    pub line: Option<i64>,
    pub original_line: Option<i64>,
    pub side: Option<String>,
    pub user: Option<GitHubOwner>,
    pub created_at: String,
    pub html_url: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubMergeResult {
    pub merged: bool,
    pub message: String,
    pub sha: Option<String>,
    #[serde(default)]
    pub completed_issue_key: Option<String>,
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
    request_class: RequestClass,
    dispatch_enabled: bool,
}

impl GitHubClient {
    pub fn new(token: impl Into<String>) -> CoreResult<Self> {
        let base = std::env::var("FORGEDESK_GITHUB_API_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_API.to_owned());
        Self::build(token, base, RequestClass::InteractiveRead, true)
    }

    fn with_base(token: impl Into<String>, base: impl Into<String>) -> CoreResult<Self> {
        Self::build(token, base, RequestClass::InteractiveRead, false)
    }

    fn build(token: impl Into<String>, base: impl Into<String>, request_class: RequestClass, dispatch_enabled: bool) -> CoreResult<Self> {
        let http = Client::builder()
            .user_agent("ForgeDesk/0.1 (+https://github.com/mindwork64/forgedesk)")
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|error| CoreError::network(format!("HTTP client failed: {error}")))?;
        Ok(Self { http, token: token.into(), base: base.into().trim_end_matches('/').to_owned(), request_class, dispatch_enabled })
    }

    fn execute(&self, method: reqwest::Method, path: &str, body: Option<&Value>) -> CoreResult<(String, RateSnapshot, Option<String>)> {
        let _gate = github_dispatcher::request_gate();
        let request_class = if method == reqwest::Method::GET { self.request_class } else { RequestClass::InteractiveWrite };
        if self.dispatch_enabled { github_dispatcher::authorize(request_class, path)?; }
        if method != reqwest::Method::GET { github_dispatcher::pace_mutation(); }

        let started = Instant::now();
        let mut request = self.http.request(method.clone(), format!("{}{}", self.base, path))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .bearer_auth(&self.token);
        if let Some(body) = body { request = request.json(body); }
        let response = match request.send() {
            Ok(response) => response,
            Err(error) => {
                if self.dispatch_enabled {
                    let _ = github_dispatcher::record_transport_failure(request_class, method.as_str(), path, body, started.elapsed(), &error.to_string());
                }
                return Err(CoreError::network(format!("GitHub request failed: {error}")));
            }
        };
        let status = response.status();
        let rate = rate_from_headers(&response);
        let link = response.headers().get("link").and_then(|value| value.to_str().ok()).map(str::to_owned);
        let retry_after_seconds = response.headers().get("retry-after").and_then(|value| value.to_str().ok()).and_then(|value| value.parse::<u64>().ok());
        let text = match response.text() {
            Ok(text) => text,
            Err(error) => {
                if self.dispatch_enabled {
                    let _ = github_dispatcher::record_transport_failure(request_class, method.as_str(), path, body, started.elapsed(), &error.to_string());
                }
                return Err(CoreError::network(format!("GitHub response failed: {error}")));
            }
        };
        if self.dispatch_enabled {
            let _ = github_dispatcher::record_response(request_class, method.as_str(), path, body, status.as_u16(), &rate, retry_after_seconds, started.elapsed(), text.len(), &text);
        }
        if !status.is_success() { return Err(map_status(status, &text, rate.remaining)); }
        Ok((text, rate, link))
    }

    pub fn list_repositories(&self) -> CoreResult<Page<GitHubRepo>> {
        self.list_repositories_page(1)
    }

    pub fn list_repositories_page(&self, page: u32) -> CoreResult<Page<GitHubRepo>> {
        self.get_page(&format!(
            "/user/repos?per_page=100&sort=updated&affiliation=owner,collaborator,organization_member&page={}",
            page.max(1)
        ))
    }

    pub fn list_repositories_bounded(
        &self,
        start_page: u32,
        max_pages: u32,
    ) -> CoreResult<(Vec<GitHubRepo>, bool, u32, Option<u32>)> {
        let start_page = start_page.max(1);
        let max_pages = max_pages.max(1);
        let mut page = start_page;
        let mut pages = 0u32;
        let mut items = Vec::new();
        loop {
            let result = self.list_repositories_page(page)?;
            pages += 1;
            items.extend(result.items);
            match result.next_page {
                Some(next) if next > page && pages < max_pages => page = next,
                Some(next) if next <= page => {
                    return Err(CoreError::network("GitHub pagination did not advance."));
                }
                Some(next) => return Ok((items, true, pages, Some(next))),
                None => return Ok((items, false, pages, None)),
            }
        }
    }

    /// Fetch one issue by its stable repository-local number.
    pub fn get_issue(&self, owner: &str, repo: &str, number: i64) -> CoreResult<GitHubIssue> {
        self.get_json(&format!(
            "/repos/{}/{}/issues/{number}",
            encode(owner),
            encode(repo)
        ))
    }

    /// Create an issue. Callers must route this through sync pacing policy.
    pub fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: &str,
        labels: &[String],
    ) -> CoreResult<GitHubIssue> {
        self.send_json(
            reqwest::Method::POST,
            &format!("/repos/{}/{}/issues", encode(owner), encode(repo)),
            &serde_json::json!({
                "title": title,
                "body": body,
                "labels": labels,
            }),
        )
    }

    /// Update explicitly supplied fields only. An empty labels vector clears labels.
    pub fn update_issue(
        &self,
        owner: &str,
        repo: &str,
        number: i64,
        update: &GitHubIssueUpdate<'_>,
    ) -> CoreResult<GitHubIssue> {
        let mut payload = serde_json::Map::new();
        if let Some(title) = update.title {
            payload.insert("title".into(), title.into());
        }
        if let Some(body) = update.body {
            payload.insert("body".into(), body.into());
        }
        if let Some(state) = update.state {
            let normalized = match state {
                "open" | "closed" => state,
                _ => {
                    return Err(CoreError::validation(
                        "state",
                        "GitHub issue state must be open or closed.",
                    ))
                }
            };
            payload.insert("state".into(), normalized.into());
        }
        if let Some(labels) = update.labels {
            payload.insert("labels".into(), serde_json::json!(labels));
        }
        if payload.is_empty() {
            return self.get_issue(owner, repo, number);
        }
        self.send_json(
            reqwest::Method::PATCH,
            &format!("/repos/{}/{}/issues/{number}", encode(owner), encode(repo)),
            &serde_json::Value::Object(payload),
        )
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
            "/repos/{owner}/{repo}/issues?state=all&sort=updated&direction=desc&per_page={}&page={}",
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

    pub fn get_repository(&self, owner: &str, repo: &str) -> CoreResult<GitHubRepo> {
        self.get_json(&format!("/repos/{}/{}", encode(owner), encode(repo)))
    }

    pub fn list_commits(
        &self,
        owner: &str,
        repo: &str,
        per_page: u32,
    ) -> CoreResult<Vec<GitHubCommit>> {
        let page: Page<GitHubCommit> = self.get_page(&format!(
            "/repos/{}/{}/commits?per_page={}",
            encode(owner),
            encode(repo),
            per_page.clamp(1, 100)
        ))?;
        Ok(page.items)
    }

    pub fn review_requested(
        &self,
        owner: &str,
        repo: &str,
        login: &str,
    ) -> CoreResult<Vec<GitHubSearchIssue>> {
        let query = format!("repo:{owner}/{repo} is:pr is:open review-requested:{login}");
        let path = format!("/search/issues?q={}&per_page=50", encode(&query));
        let result: GitHubSearchResult = self.get_json(&path)?;
        Ok(result.items)
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

    pub fn list_pull_comments(
        &self,
        owner: &str,
        repo: &str,
        number: i64,
    ) -> CoreResult<Page<GitHubReviewComment>> {
        let path = format!(
            "/repos/{}/{}/pulls/{}/comments?per_page=100",
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
        let mut page_number = 1u32;
        let mut seen = HashSet::new();
        let mut files = Vec::new();
        loop {
            if !seen.insert(page_number) {
                return Err(CoreError::network("GitHub file pagination loop detected."));
            }
            let path = format!(
                "/repos/{}/{}/pulls/{}/files?per_page=100&page={page_number}",
                encode(owner),
                encode(repo),
                number
            );
            let page: Page<GitHubFile> = self.get_page(&path)?;
            if page.items.is_empty() {
                if page.next_page.is_some() {
                    return Err(CoreError::network(
                        "GitHub returned an empty file page with a next page.",
                    ));
                }
                break;
            }
            files.extend(page.items);
            match page.next_page {
                Some(next) if next > page_number => page_number = next,
                Some(_) => {
                    return Err(CoreError::network(
                        "GitHub file pagination did not advance.",
                    ))
                }
                None => break,
            }
        }
        Ok(files)
    }

    pub fn merge_pull(
        &self,
        owner: &str,
        repo: &str,
        number: i64,
        method: &str,
        expected_sha: Option<&str>,
    ) -> CoreResult<GitHubMergeResult> {
        let mut body = serde_json::Map::new();
        body.insert("merge_method".into(), method.into());
        if let Some(sha) = expected_sha {
            body.insert("sha".into(), sha.into());
        }
        self.send_json(
            reqwest::Method::PUT,
            &format!(
                "/repos/{}/{}/pulls/{number}/merge",
                encode(owner),
                encode(repo)
            ),
            &serde_json::Value::Object(body),
        )
    }

    pub fn close_pull(&self, owner: &str, repo: &str, number: i64) -> CoreResult<GitHubPull> {
        self.send_json(
            reqwest::Method::PATCH,
            &format!("/repos/{}/{}/pulls/{number}", encode(owner), encode(repo)),
            &serde_json::json!({ "state": "closed" }),
        )
    }

    pub fn submit_review(
        &self,
        owner: &str,
        repo: &str,
        number: i64,
        event: &str,
        body: &str,
    ) -> CoreResult<GitHubReview> {
        let mut payload = serde_json::Map::new();
        payload.insert("event".into(), event.into());
        if !body.trim().is_empty() {
            payload.insert("body".into(), body.into());
        }
        self.send_json(
            reqwest::Method::POST,
            &format!(
                "/repos/{}/{}/pulls/{number}/reviews",
                encode(owner),
                encode(repo)
            ),
            &serde_json::Value::Object(payload),
        )
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

    pub fn get_authenticated_user(&self) -> CoreResult<(String, i64)> {
        #[derive(Deserialize)]
        struct AuthenticatedUser { login: String, id: i64 }
        let user: AuthenticatedUser = self.get_json("/user")?;
        Ok((user.login, user.id))
    }

    fn send_json<T: for<'de> Deserialize<'de>>(&self, method: reqwest::Method, path: &str, body: &Value) -> CoreResult<T> {
        let (text, _, _) = self.execute(method, path, Some(body))?;
        serde_json::from_str(&text).map_err(|error| CoreError::network(format!("GitHub JSON parsing failed: {error}")))
    }

    fn get_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> CoreResult<T> {
        let (text, _, _) = self.execute(reqwest::Method::GET, path, None)?;
        serde_json::from_str(&text).map_err(|error| CoreError::network(format!("GitHub JSON parsing failed: {error}")))
    }

    fn get_page<T: for<'de> Deserialize<'de>>(&self, path: &str) -> CoreResult<Page<T>> {
        let (text, rate, link) = self.execute(reqwest::Method::GET, path, None)?;
        let items: Vec<T> = serde_json::from_str(&text).map_err(|error| CoreError::network(format!("GitHub JSON parsing failed: {error}")))?;
        Ok(Page { items, next_page: link.as_deref().and_then(parse_next_page), rate })
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
    let normalized = message.to_ascii_lowercase();
    if status == StatusCode::TOO_MANY_REQUESTS || normalized.contains("secondary rate limit") || normalized.contains("abuse detection") {
        return CoreError::rate_limit_secondary(format!("GitHub secondary rate limit: {message}"));
    }
    if status == StatusCode::UNAUTHORIZED {
        return CoreError::auth(format!("GitHub rejected the token: {message}"));
    }
    if status == StatusCode::FORBIDDEN && remaining == Some(0) {
        return CoreError::rate_limit_primary(format!("GitHub rate limit exhausted: {message}"));
    }
    if status == StatusCode::FORBIDDEN {
        return CoreError::auth(format!("GitHub permission denied: {message}"));
    }
    if status == StatusCode::NOT_FOUND {
        return CoreError::not_found("GitHub object");
    }
    if status == StatusCode::CONFLICT || status == StatusCode::METHOD_NOT_ALLOWED {
        return CoreError::conflict(format!("GitHub rejected the action: {message}"));
    }
    if status == StatusCode::UNPROCESSABLE_ENTITY {
        return CoreError::validation_message(format!("GitHub validation failed: {message}"));
    }
    CoreError::network(format!("GitHub request failed: {message}"))
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    use super::*;

    fn repo_json(id: i64) -> String {
        format!(
            r#"{{"id":{id},"name":"repo{id}","full_name":"owner/repo{id}","private":false,"default_branch":"main","html_url":"https://github.com/owner/repo{id}","description":null,"updated_at":null,"owner":{{"login":"owner"}}}}"#
        )
    }

    #[test]
    fn discovery_pagination_loads_more_than_one_hundred_repositories() {
        let first = (1..=100).map(repo_json).collect::<Vec<_>>().join(",");
        let server = mock_server(vec![
            MockResponse {
                status: 200,
                body: format!("[{first}]"),
                next_page: Some(2),
            },
            MockResponse {
                status: 200,
                body: format!("[{}]", repo_json(101)),
                next_page: None,
            },
        ]);
        let client = GitHubClient::with_base("token", server.base).expect("client");
        let (repos, has_more, pages, next) =
            client.list_repositories_bounded(1, 10).expect("pages");
        assert_eq!(repos.len(), 101);
        assert!(!has_more);
        assert_eq!(pages, 2);
        assert!(next.is_none());
    }

    #[test]
    fn discovery_pagination_rejects_non_advancing_links() {
        let server = mock_server(vec![MockResponse {
            status: 200,
            body: format!("[{}]", repo_json(1)),
            next_page: Some(1),
        }]);
        let client = GitHubClient::with_base("token", server.base).expect("client");
        let error = client
            .list_repositories_bounded(1, 10)
            .expect_err("cycle must fail");
        assert_eq!(error.category(), crate::error::ErrorCategory::Network);
    }

    #[test]
    fn discovery_pagination_returns_page_failure() {
        let server = mock_server(vec![
            MockResponse {
                status: 200,
                body: format!("[{}]", repo_json(1)),
                next_page: Some(2),
            },
            MockResponse {
                status: 500,
                body: "server error".into(),
                next_page: None,
            },
        ]);
        let client = GitHubClient::with_base("token", server.base).expect("client");
        let error = client
            .list_repositories_bounded(1, 10)
            .expect_err("second page must fail");
        assert_eq!(error.category(), crate::error::ErrorCategory::Network);
    }

    #[test]
    fn github_repo_contract_uses_snake_case_in_and_camel_case_out() {
        let fixture = r#"{
          "id": 123,
          "name": "forgedesk",
          "full_name": "mindwork64/forgedesk",
          "private": false,
          "default_branch": "main",
          "html_url": "https://github.com/mindwork64/forgedesk",
          "description": "Test repository",
          "updated_at": "2026-11-15T12:00:00Z",
          "owner": { "login": "mindwork64" }
        }"#;
        let repo: GitHubRepo = serde_json::from_str(fixture).expect("GitHub response");
        assert_eq!(repo.id, 123);
        assert_eq!(repo.name, "forgedesk");
        assert_eq!(repo.full_name, "mindwork64/forgedesk");
        assert_eq!(repo.default_branch.as_deref(), Some("main"));
        assert_eq!(repo.html_url, "https://github.com/mindwork64/forgedesk");
        assert_eq!(repo.updated_at.as_deref(), Some("2026-11-15T12:00:00Z"));
        assert_eq!(repo.owner.login, "mindwork64");

        let serialized = serde_json::to_value(&repo).expect("frontend DTO");
        assert_eq!(serialized["fullName"], "mindwork64/forgedesk");
        assert_eq!(serialized["defaultBranch"], "main");
        assert_eq!(
            serialized["htmlUrl"],
            "https://github.com/mindwork64/forgedesk"
        );
        assert_eq!(serialized["updatedAt"], "2026-11-15T12:00:00Z");
        assert_eq!(serialized["owner"]["login"], "mindwork64");
        assert!(serialized.get("full_name").is_none());
        assert!(serialized.get("default_branch").is_none());
        assert!(serialized.get("html_url").is_none());
        assert!(serialized.get("updated_at").is_none());
    }

    struct MockServer {
        base: String,
    }

    struct MockResponse {
        status: u16,
        body: String,
        next_page: Option<u32>,
    }

    fn mock_server(responses: Vec<MockResponse>) -> MockServer {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let address = listener.local_addr().expect("address");
        let count = responses.len();
        thread::spawn(move || {
            for response in responses.into_iter().take(count) {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut request = [0u8; 4096];
                let _ = stream.read(&mut request);
                let reason = if response.status == 200 {
                    "OK"
                } else {
                    "Internal Server Error"
                };
                let link = response
                    .next_page
                    .map(|page| {
                        format!("Link: <http://127.0.0.1/files?page={page}>; rel=\"next\"\r\n")
                    })
                    .unwrap_or_default();
                let head = format!(
                    "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n",
                    response.status,
                    reason,
                    response.body.len(),
                    link
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(response.body.as_bytes());
            }
        });
        MockServer {
            base: format!("http://{address}"),
        }
    }

    fn file(name: &str) -> String {
        format!(
            r#"{{"filename":"{name}","status":"modified","additions":1,"deletions":0,"changes":1,"patch":null,"previous_filename":null}}"#
        )
    }

    fn body(files: &[&str]) -> String {
        format!(
            "[{}]",
            files
                .iter()
                .map(|name| file(name))
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    fn client(server: &MockServer) -> GitHubClient {
        GitHubClient::with_base("test-token", server.base.clone()).expect("client")
    }

    #[test]
    fn file_pagination_single_page_without_next() {
        let server = mock_server(vec![MockResponse {
            status: 200,
            body: body(&["a.rs", "b.rs"]),
            next_page: None,
        }]);
        let files = client(&server).list_pull_files("o", "r", 1).expect("files");
        assert_eq!(
            files
                .iter()
                .map(|f| f.filename.as_str())
                .collect::<Vec<_>>(),
            ["a.rs", "b.rs"]
        );
    }

    #[test]
    fn file_pagination_multiple_pages_preserves_order() {
        let server = mock_server(vec![
            MockResponse {
                status: 200,
                body: body(&["a.rs", "b.rs"]),
                next_page: Some(2),
            },
            MockResponse {
                status: 200,
                body: body(&["c.rs"]),
                next_page: Some(3),
            },
            MockResponse {
                status: 200,
                body: body(&["d.rs"]),
                next_page: None,
            },
        ]);
        let files = client(&server).list_pull_files("o", "r", 7).expect("files");
        assert_eq!(
            files
                .iter()
                .map(|f| f.filename.as_str())
                .collect::<Vec<_>>(),
            ["a.rs", "b.rs", "c.rs", "d.rs"]
        );
    }

    #[test]
    fn file_pagination_empty_page_stops_without_partial_error() {
        let server = mock_server(vec![
            MockResponse {
                status: 200,
                body: body(&["a.rs"]),
                next_page: Some(2),
            },
            MockResponse {
                status: 200,
                body: "[]".into(),
                next_page: None,
            },
        ]);
        let files = client(&server).list_pull_files("o", "r", 1).expect("files");
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn file_pagination_empty_page_with_next_is_an_error() {
        let server = mock_server(vec![
            MockResponse {
                status: 200,
                body: body(&["a.rs"]),
                next_page: Some(2),
            },
            MockResponse {
                status: 200,
                body: "[]".into(),
                next_page: Some(3),
            },
        ]);
        let error = client(&server)
            .list_pull_files("o", "r", 1)
            .expect_err("empty page with next must fail");
        assert_eq!(error.category(), crate::error::ErrorCategory::Network);
    }

    #[test]
    fn file_pagination_second_page_error_fails_whole_request() {
        let server = mock_server(vec![
            MockResponse {
                status: 200,
                body: body(&["a.rs"]),
                next_page: Some(2),
            },
            MockResponse {
                status: 500,
                body: r#"{"message":"boom"}"#.into(),
                next_page: None,
            },
        ]);
        let error = client(&server)
            .list_pull_files("o", "r", 1)
            .expect_err("second page must fail");
        assert_eq!(error.category(), crate::error::ErrorCategory::Network);
    }

    #[test]
    fn file_pagination_cycle_is_rejected() {
        let server = mock_server(vec![
            MockResponse {
                status: 200,
                body: body(&["a.rs"]),
                next_page: Some(2),
            },
            MockResponse {
                status: 200,
                body: body(&["b.rs"]),
                next_page: Some(1),
            },
        ]);
        let error = client(&server)
            .list_pull_files("o", "r", 1)
            .expect_err("cycle must fail");
        assert_eq!(error.category(), crate::error::ErrorCategory::Network);
    }

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
    fn permissions_require_write_or_maintain_roles() {
        assert!(!GitHubPermissions::default().can_manage());
        assert!(GitHubPermissions {
            admin: true,
            ..Default::default()
        }
        .can_manage());
        assert!(GitHubPermissions {
            maintain: true,
            ..Default::default()
        }
        .can_manage());
        assert!(GitHubPermissions {
            push: true,
            ..Default::default()
        }
        .can_manage());
        assert!(GitHubPermissions {
            pull: true,
            ..Default::default()
        }
        .can_review());
        assert!(!GitHubPermissions {
            triage: true,
            ..Default::default()
        }
        .can_manage());
    }

    #[test]
    fn maps_rate_limit_403_to_rate_limit_error() {
        let error = map_status(
            StatusCode::FORBIDDEN,
            r#"{"message":"API rate limit exceeded"}"#,
            Some(0),
        );
        assert_eq!(error.category(), crate::error::ErrorCategory::RateLimitPrimary);
    }
}
