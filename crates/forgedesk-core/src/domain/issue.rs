use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{CoreError, CoreResult};

pub const TITLE_MAX: usize = 500;
pub const DESCRIPTION_MAX: usize = 200_000;
pub const LABEL_NAME_MAX: usize = 40;
pub const COMMENT_MAX: usize = 10_000;
pub const ESTIMATE_MAX: f64 = 1_000_000.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    #[default]
    None,
    Low,
    Medium,
    High,
    Urgent,
}

impl Priority {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Urgent => "urgent",
        }
    }

    pub const fn rank(self) -> i64 {
        match self {
            Self::None => 0,
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Urgent => 4,
        }
    }

    pub fn parse(value: &str) -> CoreResult<Self> {
        match value {
            "none" => Ok(Self::None),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "urgent" => Ok(Self::Urgent),
            other => Err(CoreError::internal(format!("Unknown priority: {other}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueType {
    Bug,
    Feature,
    #[default]
    Task,
    Improvement,
    Chore,
    Research,
}

impl IssueType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bug => "bug",
            Self::Feature => "feature",
            Self::Task => "task",
            Self::Improvement => "improvement",
            Self::Chore => "chore",
            Self::Research => "research",
        }
    }

    pub fn parse(value: &str) -> CoreResult<Self> {
        match value {
            "bug" => Ok(Self::Bug),
            "feature" => Ok(Self::Feature),
            "task" => Ok(Self::Task),
            "improvement" => Ok(Self::Improvement),
            "chore" => Ok(Self::Chore),
            "research" => Ok(Self::Research),
            other => Err(CoreError::internal(format!("Unknown issue type: {other}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusCategory {
    Backlog,
    Unstarted,
    Started,
    Completed,
    Canceled,
}

impl StatusCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Backlog => "backlog",
            Self::Unstarted => "unstarted",
            Self::Started => "started",
            Self::Completed => "completed",
            Self::Canceled => "canceled",
        }
    }

    /// Categories that close an issue for progress/metrics purposes.
    pub const fn is_closed(self) -> bool {
        matches!(self, Self::Completed | Self::Canceled)
    }

    pub fn parse(value: &str) -> CoreResult<Self> {
        match value {
            "backlog" => Ok(Self::Backlog),
            "unstarted" => Ok(Self::Unstarted),
            "started" => Ok(Self::Started),
            "completed" => Ok(Self::Completed),
            "canceled" => Ok(Self::Canceled),
            other => Err(CoreError::internal(format!(
                "Unknown status category: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusColor {
    Neutral,
    Info,
    Accent,
    Success,
    Warning,
    Danger,
}

impl StatusColor {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Info => "info",
            Self::Accent => "accent",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Danger => "danger",
        }
    }

    pub fn parse(value: &str) -> CoreResult<Self> {
        match value {
            "neutral" => Ok(Self::Neutral),
            "info" => Ok(Self::Info),
            "accent" => Ok(Self::Accent),
            "success" => Ok(Self::Success),
            "warning" => Ok(Self::Warning),
            "danger" => Ok(Self::Danger),
            other => Err(CoreError::internal(format!(
                "Unknown status color: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSource {
    Local,
    Github,
    Gitlab,
    Gitea,
}

impl IssueSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Github => "github",
            Self::Gitlab => "gitlab",
            Self::Gitea => "gitea",
        }
    }

    pub fn parse(value: &str) -> CoreResult<Self> {
        match value {
            "local" => Ok(Self::Local),
            "github" => Ok(Self::Github),
            "gitlab" => Ok(Self::Gitlab),
            "gitea" => Ok(Self::Gitea),
            other => Err(CoreError::internal(format!(
                "Unknown issue source: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusSummary {
    pub id: String,
    pub name: String,
    pub category: StatusCategory,
    pub color: StatusColor,
    pub position: i64,
    pub system_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStatus {
    #[serde(flatten)]
    pub summary: StatusSummary,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub statuses: Vec<WorkflowStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: StatusColor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueSummary {
    pub id: String,
    pub key: String,
    pub key_number: i64,
    pub title: String,
    pub status: StatusSummary,
    pub priority: Priority,
    pub issue_type: IssueType,
    pub project: Option<ProjectRef>,
    pub labels: Vec<Label>,
    pub estimate: Option<f64>,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub parent_id: Option<String>,
    pub source: IssueSource,
    pub remote_url: Option<String>,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub sub_issue_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommentAuthor {
    Local,
    Remote,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: String,
    pub issue_id: String,
    pub body: String,
    pub author: CommentAuthor,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEvent {
    pub id: String,
    pub issue_id: String,
    pub kind: String,
    pub data: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RelationKind {
    Blocks,
    BlockedBy,
    Related,
    Duplicate,
    DuplicatedBy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedIssue {
    pub id: String,
    pub key: String,
    pub title: String,
    pub status: StatusSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueRelation {
    pub id: String,
    pub kind: RelationKind,
    pub issue: RelatedIssue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueDetail {
    pub summary: IssueSummary,
    pub description: String,
    pub parent: Option<IssueSummary>,
    pub children: Vec<IssueSummary>,
    pub relations: Vec<IssueRelation>,
    pub comments: Vec<Comment>,
    pub activity: Vec<ActivityEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelationInputKind {
    Blocks,
    Related,
    Duplicate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FilterField {
    Status,
    StatusCategory,
    Priority,
    Type,
    Project,
    Label,
    Estimate,
    StartDate,
    DueDate,
    CreatedAt,
    UpdatedAt,
    Title,
    Parent,
    Archived,
    Source,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FilterOperator {
    Eq,
    Neq,
    In,
    NotIn,
    Contains,
    IsEmpty,
    IsNotEmpty,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FilterNode {
    Condition {
        field: FilterField,
        operator: FilterOperator,
        #[serde(default)]
        value: Value,
    },
    Group {
        op: LogicalOp,
        conditions: Vec<FilterNode>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortField {
    UpdatedAt,
    CreatedAt,
    Priority,
    Status,
    Title,
    DueDate,
    Estimate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortSpec {
    pub field: SortField,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueQuery {
    #[serde(default)]
    pub filter: Option<FilterNode>,
    #[serde(default)]
    pub sort: Vec<SortSpec>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub include_archived: bool,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueInput {
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub status_id: Option<String>,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub issue_type: IssueType,
    pub estimate: Option<f64>,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub project_id: Option<String>,
    pub parent_id: Option<String>,
    #[serde(default)]
    pub label_names: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIssueInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status_id: Option<String>,
    pub priority: Option<Priority>,
    pub issue_type: Option<IssueType>,
    pub estimate: Option<f64>,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub project_id: Option<String>,
    pub parent_id: Option<String>,
    pub label_names: Option<Vec<String>>,
    #[serde(default)]
    pub clear_fields: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchIssueInput {
    pub status_id: Option<String>,
    pub priority: Option<Priority>,
    pub issue_type: Option<IssueType>,
    #[serde(default)]
    pub add_label_names: Vec<String>,
    #[serde(default)]
    pub remove_label_names: Vec<String>,
    pub archive: Option<bool>,
}

pub fn validate_title(raw: &str) -> CoreResult<String> {
    let title = raw.trim();
    if title.is_empty() {
        return Err(CoreError::validation("title", "Issue title is required."));
    }
    if title.chars().count() > TITLE_MAX {
        return Err(CoreError::validation(
            "title",
            format!("Issue title must be at most {TITLE_MAX} characters."),
        ));
    }
    if title.chars().any(|c| c.is_control() && c != '\t') {
        return Err(CoreError::validation(
            "title",
            "Issue title must not contain control characters.",
        ));
    }
    Ok(title.to_owned())
}

pub fn validate_description(description: &str) -> CoreResult<()> {
    if description.chars().count() > DESCRIPTION_MAX {
        return Err(CoreError::validation(
            "description",
            format!("Description must be at most {DESCRIPTION_MAX} characters."),
        ));
    }
    Ok(())
}

pub fn validate_estimate(value: f64) -> CoreResult<()> {
    if !value.is_finite() || value < 0.0 || value > ESTIMATE_MAX {
        return Err(CoreError::validation(
            "estimate",
            "Estimate must be a finite number between 0 and 1,000,000.",
        ));
    }
    Ok(())
}

pub fn validate_date(field: &str, value: &str) -> CoreResult<()> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| CoreError::validation(field, "Date must use the YYYY-MM-DD format."))?;
    Ok(())
}

pub fn validate_label_name(raw: &str) -> CoreResult<String> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(CoreError::validation("labels", "Label name is required."));
    }
    if name.chars().count() > LABEL_NAME_MAX {
        return Err(CoreError::validation(
            "labels",
            format!("Label name must be at most {LABEL_NAME_MAX} characters."),
        ));
    }
    if name.chars().any(|c| c.is_control()) {
        return Err(CoreError::validation(
            "labels",
            "Label name must not contain control characters.",
        ));
    }
    Ok(name.to_owned())
}

pub fn validate_comment(raw: &str) -> CoreResult<String> {
    let body = raw.trim();
    if body.is_empty() {
        return Err(CoreError::validation("body", "Comment cannot be empty."));
    }
    if body.chars().count() > COMMENT_MAX {
        return Err(CoreError::validation(
            "body",
            format!("Comment must be at most {COMMENT_MAX} characters."),
        ));
    }
    Ok(body.to_owned())
}

impl CreateIssueInput {
    pub fn validate(&self) -> CoreResult<()> {
        validate_title(&self.title)?;
        validate_description(&self.description)?;
        if let Some(estimate) = self.estimate {
            validate_estimate(estimate)?;
        }
        if let Some(date) = &self.start_date {
            validate_date("startDate", date)?;
        }
        if let Some(date) = &self.due_date {
            validate_date("dueDate", date)?;
        }
        for label in &self.label_names {
            validate_label_name(label)?;
        }
        if let (Some(start), Some(due)) = (&self.start_date, &self.due_date) {
            if start > due {
                return Err(CoreError::validation(
                    "dueDate",
                    "Due date cannot be earlier than the start date.",
                ));
            }
        }
        Ok(())
    }

    pub fn normalized_title(&self) -> String {
        self.title.trim().to_owned()
    }
}

impl UpdateIssueInput {
    pub const CLEARABLE_FIELDS: &'static [&'static str] =
        &["estimate", "startDate", "dueDate", "projectId", "parentId"];

    pub fn validate(&self) -> CoreResult<()> {
        if let Some(title) = &self.title {
            validate_title(title)?;
        }
        if let Some(description) = &self.description {
            validate_description(description)?;
        }
        if let Some(estimate) = self.estimate {
            validate_estimate(estimate)?;
        }
        if let Some(date) = &self.start_date {
            validate_date("startDate", date)?;
        }
        if let Some(date) = &self.due_date {
            validate_date("dueDate", date)?;
        }
        if let Some(labels) = &self.label_names {
            for label in labels {
                validate_label_name(label)?;
            }
        }
        for field in &self.clear_fields {
            if !Self::CLEARABLE_FIELDS.contains(&field.as_str()) {
                return Err(CoreError::validation(
                    "clearFields",
                    format!("Field cannot be cleared: {field}"),
                ));
            }
        }
        Ok(())
    }
}

impl IssueQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(500).clamp(1, 1000)
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }
}

/// Validation limits for filter trees. One nested group level is the v1 contract.
pub fn validate_filter_tree(node: &FilterNode) -> CoreResult<()> {
    fn walk(node: &FilterNode, depth: usize, condition_count: &mut usize) -> CoreResult<()> {
        match node {
            FilterNode::Condition { .. } => {
                *condition_count += 1;
                if *condition_count > 50 {
                    return Err(CoreError::validation(
                        "filter",
                        "A filter can contain at most 50 conditions.",
                    ));
                }
            }
            FilterNode::Group {
                conditions: children,
                ..
            } => {
                if depth >= 3 {
                    return Err(CoreError::validation(
                        "filter",
                        "Filters support at most one nested group level.",
                    ));
                }
                if children.is_empty() {
                    return Err(CoreError::validation(
                        "filter",
                        "Filter groups cannot be empty.",
                    ));
                }
                for child in children {
                    walk(child, depth + 1, condition_count)?;
                }
            }
        }
        Ok(())
    }
    let mut count = 0;
    walk(node, 1, &mut count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_issue_input() {
        let mut input = CreateIssueInput {
            title: "  Fix capture  ".into(),
            description: String::new(),
            status_id: None,
            priority: Priority::High,
            issue_type: IssueType::Bug,
            estimate: Some(3.0),
            start_date: Some("2026-11-15".into()),
            due_date: Some("2026-11-20".into()),
            project_id: None,
            parent_id: None,
            label_names: vec!["capture".into()],
        };
        input.validate().expect("valid input");
        assert_eq!(input.normalized_title(), "Fix capture");

        input.title = "   ".into();
        assert!(input.validate().is_err());

        input.title = "Valid".into();
        input.estimate = Some(-1.0);
        assert!(input.validate().is_err());

        input.estimate = None;
        input.due_date = Some("2026-11-01".into());
        assert!(input.validate().is_err());
    }

    #[test]
    fn filter_tree_depth_is_bounded() {
        let valid = FilterNode::Group {
            op: LogicalOp::And,
            conditions: vec![
                FilterNode::Condition {
                    field: FilterField::Priority,
                    operator: FilterOperator::Eq,
                    value: Value::String("high".into()),
                },
                FilterNode::Group {
                    op: LogicalOp::Or,
                    conditions: vec![FilterNode::Condition {
                        field: FilterField::StatusCategory,
                        operator: FilterOperator::Eq,
                        value: Value::String("started".into()),
                    }],
                },
            ],
        };
        validate_filter_tree(&valid).expect("valid filter");

        let invalid = FilterNode::Group {
            op: LogicalOp::And,
            conditions: vec![FilterNode::Group {
                op: LogicalOp::Or,
                conditions: vec![FilterNode::Group {
                    op: LogicalOp::And,
                    conditions: vec![FilterNode::Condition {
                        field: FilterField::Title,
                        operator: FilterOperator::Contains,
                        value: Value::String("x".into()),
                    }],
                }],
            }],
        };
        assert!(validate_filter_tree(&invalid).is_err());
    }
}
