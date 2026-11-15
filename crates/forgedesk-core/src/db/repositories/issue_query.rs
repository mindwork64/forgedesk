use rusqlite::types::Value as SqlValue;
use serde_json::Value;

use crate::domain::{
    validate_filter_tree, FilterField, FilterNode, FilterOperator, IssueQuery, LogicalOp, Priority,
    SortDirection, SortField,
};
use crate::error::{CoreError, CoreResult};

pub struct BuiltQuery {
    pub where_clause: String,
    pub order_clause: String,
    pub params: Vec<SqlValue>,
    pub limit: i64,
    pub offset: i64,
}

pub fn build(workspace_id: &str, query: &IssueQuery) -> CoreResult<BuiltQuery> {
    let mut params: Vec<SqlValue> = vec![SqlValue::Text(workspace_id.to_owned())];
    let mut clauses: Vec<String> = vec![
        "i.workspace_id = ?".to_owned(),
        "i.deleted_at IS NULL".to_owned(),
    ];

    if !query.include_archived {
        clauses.push("i.archived_at IS NULL".to_owned());
    }

    if let Some(filter) = &query.filter {
        validate_filter_tree(filter)?;
        let sql = filter_sql(filter, &mut params)?;
        clauses.push(format!("({sql})"));
    }

    if let Some(search) = query.search.as_deref().map(str::trim) {
        if !search.is_empty() {
            let fts = fts_query(search);
            params.push(SqlValue::Text(fts));
            clauses.push(
                "i.id IN (SELECT issue_id FROM issue_fts WHERE issue_fts MATCH ?)".to_owned(),
            );
        }
    }

    Ok(BuiltQuery {
        where_clause: clauses.join(" AND "),
        order_clause: order_clause(&query.sort),
        params,
        limit: query.limit(),
        offset: query.offset(),
    })
}

fn filter_sql(node: &FilterNode, params: &mut Vec<SqlValue>) -> CoreResult<String> {
    match node {
        FilterNode::Condition {
            field,
            operator,
            value,
        } => condition_sql(*field, *operator, value, params),
        FilterNode::Group { op, conditions } => {
            let joiner = match op {
                LogicalOp::And => " AND ",
                LogicalOp::Or => " OR ",
            };
            let parts = conditions
                .iter()
                .map(|child| filter_sql(child, params))
                .collect::<CoreResult<Vec<_>>>()?;
            Ok(format!("({})", parts.join(joiner)))
        }
    }
}

fn condition_sql(
    field: FilterField,
    operator: FilterOperator,
    value: &Value,
    params: &mut Vec<SqlValue>,
) -> CoreResult<String> {
    if field == FilterField::Archived {
        return archived_sql(operator, value);
    }
    if field == FilterField::Label {
        return label_sql(operator, value, params);
    }

    let expression = field_expression(field);
    match operator {
        FilterOperator::IsEmpty => Ok(format!("({expression} IS NULL)")),
        FilterOperator::IsNotEmpty => Ok(format!("({expression} IS NOT NULL)")),
        FilterOperator::Contains => {
            if field != FilterField::Title {
                return Err(CoreError::validation(
                    "filter",
                    "Only the title field supports contains.",
                ));
            }
            let text = expect_string(value)?;
            params.push(SqlValue::Text(format!("%{}%", escape_like(&text))));
            Ok("(i.title LIKE ? ESCAPE '\\')".to_owned())
        }
        FilterOperator::Eq => {
            params.push(expect_scalar(field, value)?);
            Ok(format!("({expression} = ?)"))
        }
        FilterOperator::Neq => {
            params.push(expect_scalar(field, value)?);
            Ok(format!("({expression} <> ?)"))
        }
        FilterOperator::In | FilterOperator::NotIn => {
            let values = expect_array(field, value)?;
            if values.is_empty() {
                return Err(CoreError::validation(
                    "filter",
                    "The in/notIn operators require a non-empty list.",
                ));
            }
            if values.len() > 100 {
                return Err(CoreError::validation(
                    "filter",
                    "The in/notIn operators support at most 100 values.",
                ));
            }
            let placeholders = vec!["?"; values.len()].join(", ");
            params.extend(values);
            let negation = if operator == FilterOperator::NotIn {
                "NOT"
            } else {
                ""
            };
            Ok(format!("({expression} {negation} IN ({placeholders}))"))
        }
        FilterOperator::Gt | FilterOperator::Gte | FilterOperator::Lt | FilterOperator::Lte => {
            if !matches!(
                field,
                FilterField::Estimate
                    | FilterField::StartDate
                    | FilterField::DueDate
                    | FilterField::CreatedAt
                    | FilterField::UpdatedAt
                    | FilterField::Priority
            ) {
                return Err(CoreError::validation(
                    "filter",
                    "This field does not support range comparisons.",
                ));
            }
            let sql_operator = match operator {
                FilterOperator::Gt => ">",
                FilterOperator::Gte => ">=",
                FilterOperator::Lt => "<",
                FilterOperator::Lte => "<=",
                _ => unreachable!(),
            };
            if field == FilterField::Priority {
                let priority = Priority::parse(&expect_string(value)?)?;
                params.push(SqlValue::Integer(priority.rank()));
                return Ok(format!("({} {sql_operator} ?)", priority_rank()));
            }
            params.push(expect_scalar(field, value)?);
            Ok(format!("({expression} {sql_operator} ?)"))
        }
    }
}

fn archived_sql(operator: FilterOperator, value: &Value) -> CoreResult<String> {
    match operator {
        FilterOperator::IsEmpty => Ok("(i.archived_at IS NULL)".to_owned()),
        FilterOperator::IsNotEmpty => Ok("(i.archived_at IS NOT NULL)".to_owned()),
        FilterOperator::Eq | FilterOperator::Neq => {
            let archived = value.as_bool().ok_or_else(|| {
                CoreError::validation("filter", "Archived filters require a boolean.")
            })?;
            let condition = if archived {
                "i.archived_at IS NOT NULL"
            } else {
                "i.archived_at IS NULL"
            };
            let condition = if operator == FilterOperator::Neq {
                if archived {
                    "i.archived_at IS NULL"
                } else {
                    "i.archived_at IS NOT NULL"
                }
            } else {
                condition
            };
            Ok(format!("({condition})"))
        }
        _ => Err(CoreError::validation(
            "filter",
            "Unsupported operator for the archived field.",
        )),
    }
}

fn label_sql(
    operator: FilterOperator,
    value: &Value,
    params: &mut Vec<SqlValue>,
) -> CoreResult<String> {
    let exists = "EXISTS (SELECT 1 FROM issue_label il WHERE il.issue_id = i.id";
    match operator {
        FilterOperator::IsEmpty => {
            Ok("(NOT EXISTS (SELECT 1 FROM issue_label il WHERE il.issue_id = i.id))".to_owned())
        }
        FilterOperator::IsNotEmpty => Ok(format!("{exists})")),
        FilterOperator::Eq => {
            let id = expect_string(value)?;
            params.push(SqlValue::Text(id));
            Ok(format!("{exists} AND il.label_id = ?)"))
        }
        FilterOperator::Neq => {
            let id = expect_string(value)?;
            params.push(SqlValue::Text(id));
            Ok(format!("(NOT {exists} AND il.label_id = ?))"))
        }
        FilterOperator::In | FilterOperator::NotIn => {
            let values = expect_array(FilterField::Label, value)?;
            if values.is_empty() {
                return Err(CoreError::validation(
                    "filter",
                    "The label filter requires at least one label.",
                ));
            }
            let placeholders = vec!["?"; values.len()].join(", ");
            params.extend(values);
            let negation = if operator == FilterOperator::NotIn {
                "NOT "
            } else {
                ""
            };
            Ok(format!(
                "({negation}{exists} AND il.label_id IN ({placeholders})))"
            ))
        }
        _ => Err(CoreError::validation(
            "filter",
            "Unsupported operator for the label field.",
        )),
    }
}

fn field_expression(field: FilterField) -> &'static str {
    match field {
        FilterField::Status => "i.status_id",
        FilterField::StatusCategory => "s.category",
        FilterField::Priority => "i.priority",
        FilterField::Type => "i.issue_type",
        FilterField::Project => "i.project_id",
        FilterField::Estimate => "i.estimate",
        FilterField::StartDate => "i.start_date",
        FilterField::DueDate => "i.due_date",
        FilterField::CreatedAt => "i.created_at",
        FilterField::UpdatedAt => "i.updated_at",
        FilterField::Title => "i.title",
        FilterField::Parent => "i.parent_id",
        FilterField::Source => "i.source",
        FilterField::Label | FilterField::Archived => "NULL",
    }
}

fn priority_rank() -> &'static str {
    "CASE i.priority WHEN 'urgent' THEN 4 WHEN 'high' THEN 3 WHEN 'medium' THEN 2 \
     WHEN 'low' THEN 1 ELSE 0 END"
}

fn expect_string(value: &Value) -> CoreResult<String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| CoreError::validation("filter", "Expected a text filter value."))
}

fn expect_array(field: FilterField, value: &Value) -> CoreResult<Vec<SqlValue>> {
    let array = value
        .as_array()
        .ok_or_else(|| CoreError::validation("filter", "Expected a list filter value."))?;
    array
        .iter()
        .map(|item| expect_scalar(field, item))
        .collect()
}

fn expect_scalar(field: FilterField, value: &Value) -> CoreResult<SqlValue> {
    match field {
        FilterField::Estimate => value
            .as_f64()
            .map(SqlValue::Real)
            .ok_or_else(|| CoreError::validation("filter", "Estimate filters require a number.")),
        FilterField::Status
        | FilterField::StatusCategory
        | FilterField::Type
        | FilterField::Project
        | FilterField::Label
        | FilterField::StartDate
        | FilterField::DueDate
        | FilterField::CreatedAt
        | FilterField::UpdatedAt
        | FilterField::Title
        | FilterField::Parent
        | FilterField::Source => {
            let text = expect_string(value)?;
            Ok(SqlValue::Text(text))
        }
        FilterField::Priority => {
            let text = expect_string(value)?;
            Ok(SqlValue::Text(text))
        }
        FilterField::Archived => unreachable!("archived filters are handled separately"),
    }
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn fts_query(input: &str) -> String {
    input
        .chars()
        .take(200)
        .collect::<String>()
        .split_whitespace()
        .map(|term| format!("\"{}\"", term.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" ")
}

fn order_clause(sort: &[crate::domain::SortSpec]) -> String {
    let mut parts = Vec::new();
    for spec in sort.iter().take(3) {
        let direction = match spec.direction {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        let expression = match spec.field {
            SortField::UpdatedAt => "i.updated_at",
            SortField::CreatedAt => "i.created_at",
            SortField::Priority => {
                return format!(
                    " ORDER BY {0} {1}, i.updated_at DESC, i.key_number DESC",
                    priority_rank(),
                    direction
                );
            }
            SortField::Status => "s.position",
            SortField::Title => "i.title COLLATE NOCASE",
            SortField::DueDate => "i.due_date",
            SortField::Estimate => "i.estimate",
        };
        if spec.field == SortField::DueDate {
            parts.push(format!(
                "(i.due_date IS NULL) ASC, {expression} {direction}"
            ));
        } else {
            parts.push(format!("{expression} {direction}"));
        }
    }
    if parts.is_empty() {
        parts.push("i.updated_at DESC".to_owned());
        parts.push("i.key_number DESC".to_owned());
    }
    format!(" ORDER BY {}", parts.join(", "))
}
