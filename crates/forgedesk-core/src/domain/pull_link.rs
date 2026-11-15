use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullLink {
    pub id: String,
    pub local_issue_id: String,
    pub issue_key: String,
    pub issue_title: String,
    pub provider: String,
    pub account_id: String,
    pub repository_id: String,
    pub pull_number: i64,
    pub pull_url: String,
    pub title: String,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
}
