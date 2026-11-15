use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightBucket {
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightPoint {
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightsSummary {
    pub generated_at: String,
    pub data_through: String,
    pub range_days: i64,
    pub total_created: i64,
    pub total_completed: i64,
    pub open_count: i64,
    pub overdue_count: i64,
    pub avg_cycle_days: Option<f64>,
    pub throughput: Vec<InsightPoint>,
    pub by_status: Vec<InsightBucket>,
    pub by_project: Vec<InsightBucket>,
}
