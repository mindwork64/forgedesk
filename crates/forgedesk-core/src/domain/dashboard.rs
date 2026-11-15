use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardWidget {
    pub id: String,
    pub dashboard_id: String,
    pub kind: String,
    pub title: String,
    pub width: i64,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub id: String,
    pub name: String,
    pub widgets: Vec<DashboardWidget>,
    pub updated_at: String,
}
