use serde::{Deserialize, Serialize};

use crate::domain::issue::{validate_filter_tree, FilterNode, SortSpec};
use crate::error::{CoreError, CoreResult};

pub const NAME_MAX: usize = 80;
pub const MAX_SORT: usize = 3;
pub const MAX_COLUMNS: usize = 12;

pub const ALLOWED_COLUMNS: &[&str] = &[
    "key", "title", "status", "priority", "project", "labels", "updated", "created", "estimate",
    "dueDate",
];

pub const ALLOWED_GROUP_BY: &[&str] = &["status", "priority", "project", "type"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedView {
    pub id: String,
    pub name: String,
    pub filters: Option<FilterNode>,
    pub sort: Vec<SortSpec>,
    pub group_by: Option<String>,
    pub columns: Vec<String>,
    pub density: String,
    pub is_pinned: bool,
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedViewInput {
    pub name: String,
    #[serde(default)]
    pub filters: Option<FilterNode>,
    #[serde(default)]
    pub sort: Vec<SortSpec>,
    #[serde(default)]
    pub group_by: Option<String>,
    #[serde(default)]
    pub columns: Vec<String>,
    #[serde(default)]
    pub density: Option<String>,
    #[serde(default)]
    pub is_pinned: Option<bool>,
}

impl SavedViewInput {
    pub fn validate(&self) -> CoreResult<()> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(CoreError::validation("name", "View name is required."));
        }
        if name.chars().count() > NAME_MAX {
            return Err(CoreError::validation(
                "name",
                format!("View name must be at most {NAME_MAX} characters."),
            ));
        }
        if let Some(filter) = &self.filters {
            validate_filter_tree(filter)?;
        }
        if self.sort.len() > MAX_SORT {
            return Err(CoreError::validation(
                "sort",
                format!("A view supports at most {MAX_SORT} sort fields."),
            ));
        }
        if let Some(group_by) = &self.group_by {
            if !ALLOWED_GROUP_BY.contains(&group_by.as_str()) {
                return Err(CoreError::validation(
                    "groupBy",
                    "Unsupported grouping for saved views.",
                ));
            }
        }
        if self.columns.len() > MAX_COLUMNS {
            return Err(CoreError::validation(
                "columns",
                format!("A view supports at most {MAX_COLUMNS} columns."),
            ));
        }
        for column in &self.columns {
            if !ALLOWED_COLUMNS.contains(&column.as_str()) {
                return Err(CoreError::validation(
                    "columns",
                    format!("Unsupported issue column: {column}"),
                ));
            }
        }
        if let Some(density) = &self.density {
            if density != "standard" && density != "compact" {
                return Err(CoreError::validation(
                    "density",
                    "Density must be standard or compact.",
                ));
            }
        }
        Ok(())
    }

    pub fn normalized_name(&self) -> String {
        self.name.trim().to_owned()
    }

    pub fn density(&self) -> String {
        self.density
            .clone()
            .unwrap_or_else(|| "standard".to_owned())
    }
}

pub fn default_columns() -> Vec<String> {
    vec![
        "key".into(),
        "title".into(),
        "status".into(),
        "priority".into(),
        "project".into(),
        "labels".into(),
        "updated".into(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> SavedViewInput {
        SavedViewInput {
            name: "My view".into(),
            filters: None,
            sort: vec![],
            group_by: Some("status".into()),
            columns: default_columns(),
            density: Some("standard".into()),
            is_pinned: Some(true),
        }
    }

    #[test]
    fn validates_saved_view_input() {
        input().validate().expect("valid");
        let mut invalid = input();
        invalid.name = "  ".into();
        assert!(invalid.validate().is_err());
        let mut invalid = input();
        invalid.group_by = Some("assignee".into());
        assert!(invalid.validate().is_err());
        let mut invalid = input();
        invalid.columns.push("secret".into());
        assert!(invalid.validate().is_err());
        let mut invalid = input();
        invalid.density = Some("tiny".into());
        assert!(invalid.validate().is_err());
    }
}
