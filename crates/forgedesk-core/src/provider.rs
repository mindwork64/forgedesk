use std::time::Duration;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{CoreError, CoreResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Gitlab,
    Gitea,
}

impl ProviderKind {
    pub fn parse(value: &str) -> CoreResult<Self> {
        match value {
            "gitlab" => Ok(Self::Gitlab),
            "gitea" => Ok(Self::Gitea),
            _ => Err(CoreError::validation("provider", "Unsupported provider.")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCapabilities {
    pub repositories: bool,
    pub issues: bool,
    pub merge_requests: bool,
    pub comments: bool,
    pub reviews: bool,
    pub checks: bool,
    pub write: bool,
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteRepository {
    pub id: String,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub default_branch: Option<String>,
    pub html_url: String,
    pub description: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteIssue {
    pub id: String,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<String>,
    pub html_url: String,
    pub updated_at: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteMergeRequest {
    pub id: String,
    pub number: i64,
    pub title: String,
    pub state: String,
    pub draft: bool,
    pub author: Option<String>,
    pub source_branch: String,
    pub target_branch: String,
    pub html_url: String,
    pub updated_at: Option<String>,
    pub merged_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderPage<T> {
    pub items: Vec<T>,
    pub next_page: Option<u32>,
}

pub trait ProviderAdapter {
    fn kind(&self) -> ProviderKind;
    fn capabilities(&self) -> ProviderCapabilities;
    fn list_repositories(
        &self,
        page: u32,
        per_page: u32,
    ) -> CoreResult<ProviderPage<RemoteRepository>>;
    fn list_issues(
        &self,
        repository: &str,
        page: u32,
        per_page: u32,
    ) -> CoreResult<ProviderPage<RemoteIssue>>;
    fn list_merge_requests(
        &self,
        repository: &str,
        page: u32,
        per_page: u32,
    ) -> CoreResult<ProviderPage<RemoteMergeRequest>>;
}

pub fn capabilities(kind: ProviderKind) -> ProviderCapabilities {
    let _ = kind;
    ProviderCapabilities {
        repositories: true,
        issues: true,
        merge_requests: true,
        comments: false,
        reviews: false,
        checks: false,
        write: false,
        read_only: true,
    }
}

pub struct GitLabAdapter {
    http: Client,
    base: String,
    token: String,
}

impl GitLabAdapter {
    pub fn new(base_url: &str, token: &str) -> CoreResult<Self> {
        Ok(Self {
            http: http_client()?,
            base: base_url.trim_end_matches('/').to_owned(),
            token: token.to_owned(),
        })
    }

    fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> CoreResult<(T, Option<u32>)> {
        let response = self
            .http
            .get(format!("{}/api/v4{path}", self.base))
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .map_err(network_error)?;
        let next = response
            .headers()
            .get("x-next-page")
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.is_empty())
            .and_then(|value| value.parse().ok());
        let status = response.status();
        let text = response.text().map_err(network_error)?;
        if !status.is_success() {
            return Err(status_error(status.as_u16(), &text));
        }
        Ok((serde_json::from_str(&text).map_err(json_error)?, next))
    }
}

impl ProviderAdapter for GitLabAdapter {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Gitlab
    }

    fn capabilities(&self) -> ProviderCapabilities {
        capabilities(self.kind())
    }

    fn list_repositories(
        &self,
        page: u32,
        per_page: u32,
    ) -> CoreResult<ProviderPage<RemoteRepository>> {
        let (projects, next): (Vec<GitLabProject>, _) = self.get(&format!(
            "/projects?membership=true&simple=true&per_page={}&page={}",
            per_page.clamp(1, 100),
            page.max(1)
        ))?;
        Ok(ProviderPage {
            items: projects.into_iter().map(Into::into).collect(),
            next_page: next,
        })
    }

    fn list_issues(
        &self,
        repository: &str,
        page: u32,
        per_page: u32,
    ) -> CoreResult<ProviderPage<RemoteIssue>> {
        let (issues, next): (Vec<GitLabIssue>, _) = self.get(&format!(
            "/projects/{}/issues?state=all&per_page={}&page={}",
            encode(repository),
            per_page.clamp(1, 100),
            page.max(1)
        ))?;
        Ok(ProviderPage {
            items: issues.into_iter().map(Into::into).collect(),
            next_page: next,
        })
    }

    fn list_merge_requests(
        &self,
        repository: &str,
        page: u32,
        per_page: u32,
    ) -> CoreResult<ProviderPage<RemoteMergeRequest>> {
        let (items, next): (Vec<GitLabMergeRequest>, _) = self.get(&format!(
            "/projects/{}/merge_requests?state=all&per_page={}&page={}",
            encode(repository),
            per_page.clamp(1, 100),
            page.max(1)
        ))?;
        Ok(ProviderPage {
            items: items.into_iter().map(Into::into).collect(),
            next_page: next,
        })
    }
}

pub struct GiteaAdapter {
    http: Client,
    base: String,
    token: String,
}

impl GiteaAdapter {
    pub fn new(base_url: &str, token: &str) -> CoreResult<Self> {
        Ok(Self {
            http: http_client()?,
            base: base_url.trim_end_matches('/').to_owned(),
            token: token.to_owned(),
        })
    }

    fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> CoreResult<(T, bool)> {
        let response = self
            .http
            .get(format!("{}/api/v1{path}", self.base))
            .header("Authorization", format!("token {}", self.token))
            .send()
            .map_err(network_error)?;
        let status = response.status();
        let has_more = response
            .headers()
            .get("x-hasmore")
            .and_then(|value| value.to_str().ok())
            .map(|value| value == "true")
            .unwrap_or(false);
        let text = response.text().map_err(network_error)?;
        if !status.is_success() {
            return Err(status_error(status.as_u16(), &text));
        }
        Ok((serde_json::from_str(&text).map_err(json_error)?, has_more))
    }
}

impl ProviderAdapter for GiteaAdapter {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Gitea
    }

    fn capabilities(&self) -> ProviderCapabilities {
        capabilities(self.kind())
    }

    fn list_repositories(
        &self,
        page: u32,
        per_page: u32,
    ) -> CoreResult<ProviderPage<RemoteRepository>> {
        let page = page.max(1);
        let (repos, has_more): (Vec<GiteaRepository>, _) = self.get(&format!(
            "/user/repos?page={page}&limit={}",
            per_page.clamp(1, 100)
        ))?;
        Ok(ProviderPage {
            items: repos.into_iter().map(Into::into).collect(),
            next_page: has_more.then_some(page + 1),
        })
    }

    fn list_issues(
        &self,
        repository: &str,
        page: u32,
        per_page: u32,
    ) -> CoreResult<ProviderPage<RemoteIssue>> {
        let page = page.max(1);
        let (items, has_more): (Vec<GiteaIssue>, _) = self.get(&format!(
            "/repos/{repository}/issues?state=all&type=issues&page={page}&limit={}",
            per_page.clamp(1, 100)
        ))?;
        Ok(ProviderPage {
            items: items
                .into_iter()
                .filter(|issue| issue.pull_request.is_none())
                .map(Into::into)
                .collect(),
            next_page: has_more.then_some(page + 1),
        })
    }

    fn list_merge_requests(
        &self,
        repository: &str,
        page: u32,
        per_page: u32,
    ) -> CoreResult<ProviderPage<RemoteMergeRequest>> {
        let page = page.max(1);
        let (items, has_more): (Vec<GiteaPull>, _) = self.get(&format!(
            "/repos/{repository}/pulls?state=all&page={page}&limit={}",
            per_page.clamp(1, 100)
        ))?;
        Ok(ProviderPage {
            items: items.into_iter().map(Into::into).collect(),
            next_page: has_more.then_some(page + 1),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct GitLabProject {
    id: i64,
    name: String,
    path_with_namespace: String,
    private: bool,
    default_branch: Option<String>,
    web_url: String,
    description: Option<String>,
    last_activity_at: Option<String>,
}

impl From<GitLabProject> for RemoteRepository {
    fn from(value: GitLabProject) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            full_name: value.path_with_namespace,
            private: value.private,
            default_branch: value.default_branch,
            html_url: value.web_url,
            description: value.description,
            updated_at: value.last_activity_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct GitLabIssue {
    id: i64,
    iid: i64,
    title: String,
    description: Option<String>,
    state: String,
    #[serde(default)]
    labels: Vec<String>,
    web_url: String,
    updated_at: Option<String>,
    author: Option<GitLabUser>,
}

#[derive(Debug, Clone, Deserialize)]
struct GitLabUser {
    username: String,
}

impl From<GitLabIssue> for RemoteIssue {
    fn from(value: GitLabIssue) -> Self {
        Self {
            id: value.id.to_string(),
            number: value.iid,
            title: value.title,
            body: value.description,
            state: normalize_state(&value.state),
            labels: value.labels,
            html_url: value.web_url,
            updated_at: value.updated_at,
            author: value.author.map(|user| user.username),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct GitLabMergeRequest {
    id: i64,
    iid: i64,
    title: String,
    state: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    work_in_progress: bool,
    author: Option<GitLabUser>,
    source_branch: String,
    target_branch: String,
    web_url: String,
    updated_at: Option<String>,
    merged_at: Option<String>,
}

impl From<GitLabMergeRequest> for RemoteMergeRequest {
    fn from(value: GitLabMergeRequest) -> Self {
        Self {
            id: value.id.to_string(),
            number: value.iid,
            title: value.title,
            state: normalize_state(&value.state),
            draft: value.draft || value.work_in_progress,
            author: value.author.map(|user| user.username),
            source_branch: value.source_branch,
            target_branch: value.target_branch,
            html_url: value.web_url,
            updated_at: value.updated_at,
            merged_at: value.merged_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct GiteaRepository {
    id: i64,
    name: String,
    full_name: String,
    private: bool,
    default_branch: Option<String>,
    html_url: String,
    description: Option<String>,
    updated_at: Option<String>,
}

impl From<GiteaRepository> for RemoteRepository {
    fn from(value: GiteaRepository) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            full_name: value.full_name,
            private: value.private,
            default_branch: value.default_branch,
            html_url: value.html_url,
            description: value.description,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct GiteaIssue {
    id: i64,
    number: i64,
    title: String,
    body: Option<String>,
    state: String,
    #[serde(default)]
    labels: Vec<GiteaLabel>,
    html_url: String,
    updated_at: Option<String>,
    user: Option<GiteaUser>,
    pull_request: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct GiteaLabel {
    name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GiteaUser {
    login: String,
}

impl From<GiteaIssue> for RemoteIssue {
    fn from(value: GiteaIssue) -> Self {
        Self {
            id: value.id.to_string(),
            number: value.number,
            title: value.title,
            body: value.body,
            state: normalize_state(&value.state),
            labels: value.labels.into_iter().map(|label| label.name).collect(),
            html_url: value.html_url,
            updated_at: value.updated_at,
            author: value.user.map(|user| user.login),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct GiteaPull {
    id: i64,
    number: i64,
    title: String,
    state: String,
    #[serde(default)]
    draft: bool,
    user: Option<GiteaUser>,
    head: GiteaRef,
    base: GiteaRef,
    html_url: String,
    updated_at: Option<String>,
    merged_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GiteaRef {
    #[serde(rename = "ref")]
    reference: String,
}

impl From<GiteaPull> for RemoteMergeRequest {
    fn from(value: GiteaPull) -> Self {
        Self {
            id: value.id.to_string(),
            number: value.number,
            title: value.title,
            state: normalize_state(&value.state),
            draft: value.draft,
            author: value.user.map(|user| user.login),
            source_branch: value.head.reference,
            target_branch: value.base.reference,
            html_url: value.html_url,
            updated_at: value.updated_at,
            merged_at: value.merged_at,
        }
    }
}

fn normalize_state(value: &str) -> String {
    match value {
        "opened" => "open".into(),
        other => other.into(),
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

fn http_client() -> CoreResult<Client> {
    Client::builder()
        .user_agent("ForgeDesk/0.1 (+https://github.com/mindwork64/forgedesk)")
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| CoreError::network(format!("HTTP client failed: {error}")))
}

fn network_error(error: reqwest::Error) -> CoreError {
    CoreError::network(format!("Provider request failed: {error}"))
}

fn json_error(error: serde_json::Error) -> CoreError {
    CoreError::network(format!("Provider JSON parsing failed: {error}"))
}

fn status_error(status: u16, body: &str) -> CoreError {
    let message = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("message")
                .or_else(|| value.get("error"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| format!("HTTP {status}"));
    match status {
        401 | 403 => CoreError::auth(format!("Provider rejected the token: {message}")),
        404 => CoreError::not_found("Provider object"),
        409 | 405 => CoreError::conflict(format!("Provider rejected the action: {message}")),
        422 => CoreError::validation_message(format!("Provider validation failed: {message}")),
        _ => CoreError::network(format!("Provider request failed: {message}")),
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    use super::*;

    struct MockResponse {
        body: String,
        headers: Vec<(&'static str, &'static str)>,
    }

    fn mock_server(responses: Vec<MockResponse>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let address = listener.local_addr().expect("address");
        thread::spawn(move || {
            for response in responses {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut request = [0u8; 4096];
                let _ = stream.read(&mut request);
                let headers = response
                    .headers
                    .iter()
                    .map(|(name, value)| format!("{name}: {value}\r\n"))
                    .collect::<String>();
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n",
                    response.body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(response.body.as_bytes());
            }
        });
        format!("http://{address}")
    }

    #[test]
    fn gitlab_contract_normalizes_and_paginates() {
        let base = mock_server(vec![
            MockResponse {
                body: r#"[{"id":1,"name":"Repo","path_with_namespace":"group/repo","private":false,"default_branch":"main","web_url":"https://gitlab.example/group/repo","description":"d","last_activity_at":"2026-01-01T00:00:00Z"}]"#.into(),
                headers: vec![("X-Next-Page", "2")],
            },
            MockResponse {
                body: r#"[{"id":2,"iid":4,"title":"Issue","description":"body","state":"opened","labels":["bug"],"web_url":"u","updated_at":"2026-01-01T00:00:00Z","author":{"username":"alice"}}]"#.into(),
                headers: vec![],
            },
            MockResponse {
                body: r#"[{"id":3,"iid":5,"title":"MR","state":"opened","draft":true,"work_in_progress":false,"author":{"username":"bob"},"source_branch":"feat","target_branch":"main","web_url":"m","updated_at":"2026-01-02T00:00:00Z","merged_at":null}]"#.into(),
                headers: vec![],
            },
        ]);
        let adapter = GitLabAdapter::new(&base, "token").expect("adapter");
        assert_eq!(adapter.kind(), ProviderKind::Gitlab);
        assert!(adapter.capabilities().read_only);
        assert!(adapter.capabilities().merge_requests);
        let repos = adapter.list_repositories(1, 100).expect("repos");
        assert_eq!(repos.items[0].full_name, "group/repo");
        assert_eq!(repos.next_page, Some(2));
        let issues = adapter.list_issues("group/repo", 1, 100).expect("issues");
        assert_eq!(issues.items[0].state, "open");
        assert_eq!(issues.items[0].author.as_deref(), Some("alice"));
        let mrs = adapter
            .list_merge_requests("group/repo", 1, 100)
            .expect("mrs");
        assert!(mrs.items[0].draft);
        assert_eq!(mrs.items[0].number, 5);
    }

    #[test]
    fn gitea_contract_normalizes_and_paginates() {
        let base = mock_server(vec![
            MockResponse {
                body: r#"[{"id":1,"name":"repo","full_name":"owner/repo","private":true,"default_branch":"main","html_url":"h","description":null,"updated_at":"2026-01-01T00:00:00Z"}]"#.into(),
                headers: vec![("X-HasMore", "true")],
            },
            MockResponse {
                body: r#"[{"id":2,"number":3,"title":"Issue","body":"b","state":"open","labels":[{"name":"bug"}],"html_url":"i","updated_at":"2026-01-01T00:00:00Z","user":{"login":"alice"},"pull_request":null},{"id":9,"number":4,"title":"PR","body":null,"state":"open","labels":[],"html_url":"p","updated_at":null,"user":null,"pull_request":{}}]"#.into(),
                headers: vec![],
            },
            MockResponse {
                body: r#"[{"id":3,"number":7,"title":"PR","state":"open","draft":true,"user":{"login":"bob"},"head":{"ref":"feat"},"base":{"ref":"main"},"html_url":"p","updated_at":null,"merged_at":null}]"#.into(),
                headers: vec![],
            },
        ]);
        let adapter = GiteaAdapter::new(&base, "token").expect("adapter");
        assert_eq!(adapter.kind(), ProviderKind::Gitea);
        let repos = adapter.list_repositories(1, 100).expect("repos");
        assert_eq!(repos.items[0].full_name, "owner/repo");
        assert_eq!(repos.next_page, Some(2));
        let issues = adapter.list_issues("owner/repo", 1, 100).expect("issues");
        assert_eq!(issues.items.len(), 1, "pull requests must be filtered");
        assert_eq!(issues.items[0].labels, ["bug"]);
        let mrs = adapter
            .list_merge_requests("owner/repo", 1, 100)
            .expect("mrs");
        assert!(mrs.items[0].draft);
        assert_eq!(mrs.items[0].source_branch, "feat");
    }

    #[test]
    fn capabilities_are_read_only_for_both_adapters() {
        for kind in [ProviderKind::Gitlab, ProviderKind::Gitea] {
            let caps = capabilities(kind);
            assert!(caps.repositories && caps.issues && caps.merge_requests);
            assert!(caps.read_only);
            assert!(!caps.write && !caps.comments && !caps.reviews && !caps.checks);
        }
    }
}
