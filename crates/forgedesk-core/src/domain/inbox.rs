use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxEvent {
    pub id: String,
    pub source: String,
    pub kind: String,
    pub title: String,
    pub summary: String,
    pub entity_type: String,
    pub entity_id: String,
    pub entity_key: String,
    pub entity_title: String,
    pub created_at: String,
    pub read: bool,
    pub archived: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InboxFilter {
    All,
    Unread,
    Github,
    Local,
    Sync,
}

impl InboxFilter {
    pub fn parse(raw: &str) -> Self {
        match raw {
            "unread" => Self::Unread,
            "github" => Self::Github,
            "local" => Self::Local,
            "sync" => Self::Sync,
            _ => Self::All,
        }
    }
}
