use serde::{Deserialize, Serialize};

use crate::domain::issue::{validate_date, IssueSummary, Priority, StatusSummary};
use crate::error::{CoreError, CoreResult};

pub const PLANNING_NAME_MAX: usize = 120;
pub const DOCUMENT_TITLE_MAX: usize = 200;
pub const DOCUMENT_BODY_MAX: usize = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlanningStatus {
    Planned,
    Active,
    Paused,
    Completed,
    Canceled,
}

impl PlanningStatus {
    pub fn parse(raw: &str) -> CoreResult<Self> {
        match raw {
            "planned" => Ok(Self::Planned),
            "active" => Ok(Self::Active),
            "paused" => Ok(Self::Paused),
            "completed" => Ok(Self::Completed),
            "canceled" => Ok(Self::Canceled),
            _ => Err(CoreError::internal(format!(
                "Unknown planning status: {raw}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MilestoneStatus {
    Planned,
    Active,
    Completed,
    Canceled,
}

impl MilestoneStatus {
    pub fn parse(raw: &str) -> CoreResult<Self> {
        match raw {
            "planned" => Ok(Self::Planned),
            "active" => Ok(Self::Active),
            "completed" => Ok(Self::Completed),
            "canceled" => Ok(Self::Canceled),
            _ => Err(CoreError::internal(format!(
                "Unknown milestone status: {raw}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CycleStatus {
    Upcoming,
    Active,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub completed: i64,
    pub total: i64,
    pub percent: f64,
}

impl Progress {
    /// Progress excludes canceled issues from the denominator. Completed issues
    /// are counted in both numerator and denominator. Empty projects report 0%.
    pub fn new(completed: i64, total: i64, canceled: i64) -> Self {
        let eligible = (total - canceled).max(0);
        let completed = completed.clamp(0, eligible);
        let percent = if eligible == 0 {
            0.0
        } else {
            (completed as f64 / eligible as f64) * 100.0
        };
        Self {
            completed,
            total: eligible,
            percent: (percent * 10.0).round() / 10.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitiativeRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: PlanningStatus,
    pub priority: Priority,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub initiative: Option<InitiativeRef>,
    pub progress: Progress,
    pub color: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: Option<PlanningStatus>,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub target_date: Option<String>,
    #[serde(default)]
    pub initiative_id: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub status: Option<PlanningStatus>,
    #[serde(default)]
    pub initiative_id: Option<String>,
    #[serde(default)]
    pub include_archived: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusCount {
    pub status: StatusSummary,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetail {
    pub project: Project,
    pub milestones: Vec<Milestone>,
    pub issue_counts: Vec<StatusCount>,
    pub issue_count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Milestone {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: String,
    pub due_date: Option<String>,
    pub status: MilestoneStatus,
    pub progress: Progress,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub status: Option<MilestoneStatus>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Initiative {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: PlanningStatus,
    pub priority: Priority,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub progress: Progress,
    pub project_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitiativeInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: Option<PlanningStatus>,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub target_date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitiativeDetail {
    pub initiative: Initiative,
    pub projects: Vec<Project>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleSummary {
    pub issue_count: i64,
    pub completed_count: i64,
    pub total_estimate: f64,
    pub completed_estimate: f64,
    pub carry_over_count: i64,
    pub days_remaining: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cycle {
    pub id: String,
    pub name: String,
    pub start_date: String,
    pub end_date: String,
    pub status: CycleStatus,
    pub summary: CycleSummary,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleInput {
    pub name: String,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleDetail {
    pub cycle: Cycle,
    pub issues: Vec<IssueSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: String,
    pub title: String,
    pub body: String,
    pub project_id: Option<String>,
    pub initiative_id: Option<String>,
    pub project_name: Option<String>,
    pub initiative_name: Option<String>,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInput {
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub initiative_id: Option<String>,
}

pub fn validate_name(field: &str, raw: &str) -> CoreResult<String> {
    validate_text(field, raw, 1, PLANNING_NAME_MAX)
}

pub fn validate_document_title(raw: &str) -> CoreResult<String> {
    validate_text("title", raw, 1, DOCUMENT_TITLE_MAX)
}

pub fn validate_body(raw: &str) -> CoreResult<String> {
    if raw.chars().count() > DOCUMENT_BODY_MAX {
        return Err(CoreError::validation(
            "body",
            format!("Document body must be at most {DOCUMENT_BODY_MAX} characters."),
        ));
    }
    Ok(raw.to_owned())
}

fn validate_text(field: &str, raw: &str, min: usize, max: usize) -> CoreResult<String> {
    let value = raw.trim();
    let length = value.chars().count();
    if length < min || length > max {
        return Err(CoreError::validation(
            field,
            format!("{field} must be between {min} and {max} characters."),
        ));
    }
    Ok(value.to_owned())
}

pub fn validate_date_range(start: Option<&str>, target: Option<&str>) -> CoreResult<()> {
    if let Some(start) = start {
        validate_date("startDate", start)?;
    }
    if let Some(target) = target {
        validate_date("endDate", target)?;
    }
    if let (Some(start), Some(target)) = (start, target) {
        if start > target {
            return Err(CoreError::validation(
                "endDate",
                "The end date must not be before the start date.",
            ));
        }
    }
    Ok(())
}

pub fn validate_cycle_dates(start: &str, end: &str) -> CoreResult<()> {
    validate_date("startDate", start)?;
    validate_date("endDate", end)?;
    if start > end {
        return Err(CoreError::validation(
            "endDate",
            "Cycle end date must not be before the start date.",
        ));
    }
    Ok(())
}

pub fn markdown_export(document: &Document) -> String {
    format!("# {}\n\n{}\n", document.title, document.body)
}
