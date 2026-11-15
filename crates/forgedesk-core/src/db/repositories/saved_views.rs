use rusqlite::{Connection, OptionalExtension};

use crate::domain::issue::{FilterNode, SortSpec};
use crate::domain::saved_view::{SavedView, SavedViewInput, ALLOWED_COLUMNS};
use crate::domain::time::now_rfc3339;
use crate::error::{CoreError, CoreResult};

const SELECT: &str = "SELECT id, name, filters, sort, group_by, columns, density, \
                      is_pinned, position, created_at, updated_at FROM saved_view";

pub fn list(conn: &Connection, workspace_id: &str) -> CoreResult<Vec<SavedView>> {
    let mut statement = conn.prepare(&format!(
        "{SELECT} WHERE workspace_id = ?1 ORDER BY is_pinned DESC, position ASC, name ASC"
    ))?;
    let rows = statement.query_map([workspace_id], read_row)?;
    let mut views = Vec::new();
    for row in rows {
        views.push(row??);
    }
    Ok(views)
}

pub fn create(
    conn: &Connection,
    workspace_id: &str,
    input: &SavedViewInput,
) -> CoreResult<SavedView> {
    input.validate()?;
    let now = now_rfc3339();
    let id = uuid::Uuid::now_v7().to_string();
    let position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM saved_view WHERE workspace_id = ?1",
        [workspace_id],
        |row| row.get(0),
    )?;
    let filters = encode_filters(&input.filters)?;
    let sort = encode_sort(&input.sort)?;
    let columns = encode_columns(&input.columns)?;

    conn.execute(
        "INSERT INTO saved_view \
         (id, workspace_id, name, entity, filters, sort, group_by, columns, density, \
          is_pinned, position, created_at, updated_at) \
         VALUES (?1, ?2, ?3, 'issue', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
        rusqlite::params![
            id,
            workspace_id,
            input.normalized_name(),
            filters,
            sort,
            input.group_by,
            columns,
            input.density(),
            i64::from(input.is_pinned.unwrap_or(false)),
            position,
            now,
        ],
    )?;
    find(conn, workspace_id, &id)
}

pub fn update(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    input: &SavedViewInput,
) -> CoreResult<SavedView> {
    input.validate()?;
    let changed = conn.execute(
        "UPDATE saved_view SET name = ?1, filters = ?2, sort = ?3, group_by = ?4, \
         columns = ?5, density = ?6, is_pinned = ?7, updated_at = ?8 \
         WHERE id = ?9 AND workspace_id = ?10",
        rusqlite::params![
            input.normalized_name(),
            encode_filters(&input.filters)?,
            encode_sort(&input.sort)?,
            input.group_by,
            encode_columns(&input.columns)?,
            input.density(),
            i64::from(input.is_pinned.unwrap_or(false)),
            now_rfc3339(),
            id,
            workspace_id,
        ],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Saved view"));
    }
    find(conn, workspace_id, id)
}

pub fn delete(conn: &Connection, workspace_id: &str, id: &str) -> CoreResult<()> {
    let changed = conn.execute(
        "DELETE FROM saved_view WHERE id = ?1 AND workspace_id = ?2",
        rusqlite::params![id, workspace_id],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Saved view"));
    }
    Ok(())
}

fn find(conn: &Connection, workspace_id: &str, id: &str) -> CoreResult<SavedView> {
    match conn
        .query_row(
            &format!("{SELECT} WHERE workspace_id = ?1 AND id = ?2"),
            rusqlite::params![workspace_id, id],
            read_row,
        )
        .optional()?
    {
        Some(parsed) => parsed,
        None => Err(CoreError::not_found("Saved view")),
    }
}

fn read_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CoreResult<SavedView>> {
    let raw = (
        row.get::<_, String>(0)?,
        row.get::<_, String>(1)?,
        row.get::<_, Option<String>>(2)?,
        row.get::<_, String>(3)?,
        row.get::<_, Option<String>>(4)?,
        row.get::<_, String>(5)?,
        row.get::<_, String>(6)?,
        row.get::<_, i64>(7)?,
        row.get::<_, i64>(8)?,
        row.get::<_, String>(9)?,
        row.get::<_, String>(10)?,
    );

    Ok((|| {
        let (
            id,
            name,
            filters,
            sort,
            group_by,
            columns,
            density,
            is_pinned,
            position,
            created_at,
            updated_at,
        ) = raw;
        let filters: Option<FilterNode> = match filters {
            Some(raw) => Some(serde_json::from_str(&raw).map_err(|error| {
                CoreError::internal(format!("Saved view filter is unreadable: {error}"))
            })?),
            None => None,
        };
        let sort: Vec<SortSpec> = serde_json::from_str(&sort).map_err(|error| {
            CoreError::internal(format!("Saved view sort is unreadable: {error}"))
        })?;
        let columns: Vec<String> = serde_json::from_str(&columns).map_err(|error| {
            CoreError::internal(format!("Saved view columns are unreadable: {error}"))
        })?;
        if !columns
            .iter()
            .all(|column| ALLOWED_COLUMNS.contains(&column.as_str()))
        {
            return Err(CoreError::internal(
                "Saved view contains an unknown column.",
            ));
        }
        Ok(SavedView {
            id,
            name,
            filters,
            sort,
            group_by,
            columns,
            density,
            is_pinned: is_pinned != 0,
            position,
            created_at,
            updated_at,
        })
    })())
}

fn encode_filters(filters: &Option<FilterNode>) -> CoreResult<Option<String>> {
    filters
        .as_ref()
        .map(|value| {
            serde_json::to_string(value)
                .map_err(|error| CoreError::internal(format!("Filter encoding failed: {error}")))
        })
        .transpose()
}

fn encode_sort(sort: &[SortSpec]) -> CoreResult<String> {
    serde_json::to_string(sort)
        .map_err(|error| CoreError::internal(format!("Sort encoding failed: {error}")))
}

fn encode_columns(columns: &[String]) -> CoreResult<String> {
    serde_json::to_string(columns)
        .map_err(|error| CoreError::internal(format!("Columns encoding failed: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::domain::issue::{FilterField, FilterNode, FilterOperator, SortDirection, SortField};

    fn fixture() -> (tempfile::TempDir, Database, String) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("open");
        let workspace = db
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "Views"))
            .expect("workspace");
        (dir, db, workspace.id)
    }

    fn input(name: &str) -> SavedViewInput {
        SavedViewInput {
            name: name.into(),
            filters: Some(FilterNode::Condition {
                field: FilterField::Priority,
                operator: FilterOperator::Gte,
                value: serde_json::json!("high"),
            }),
            sort: vec![SortSpec {
                field: SortField::Priority,
                direction: SortDirection::Desc,
            }],
            group_by: Some("status".into()),
            columns: crate::domain::saved_view::default_columns(),
            density: Some("standard".into()),
            is_pinned: Some(true),
        }
    }

    #[test]
    fn creates_lists_updates_and_deletes_views() {
        let (_dir, db, workspace_id) = fixture();
        let first = db
            .with_conn(|conn| create(conn, &workspace_id, &input("Pinned")))
            .expect("create first");
        let second = db
            .with_conn(|conn| {
                let mut value = input("Second");
                value.is_pinned = Some(false);
                create(conn, &workspace_id, &value)
            })
            .expect("create second");

        let views = db
            .with_conn(|conn| list(conn, &workspace_id))
            .expect("list");
        assert_eq!(views.len(), 2);
        assert_eq!(views[0].id, first.id, "pinned view sorts first");
        assert_eq!(views[1].id, second.id);
        assert_eq!(views[0].filters, first.filters);
        assert_eq!(views[0].sort, first.sort);

        let updated = db
            .with_conn(|conn| {
                let mut value = input("Renamed");
                value.is_pinned = Some(false);
                update(conn, &workspace_id, &first.id, &value)
            })
            .expect("update");
        assert_eq!(updated.name, "Renamed");
        assert!(!updated.is_pinned);

        db.with_conn(|conn| delete(conn, &workspace_id, &first.id))
            .expect("delete");
        let views = db
            .with_conn(|conn| list(conn, &workspace_id))
            .expect("list after delete");
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].id, second.id);
    }

    #[test]
    fn rejects_invalid_views_and_is_workspace_scoped() {
        let (_dir, db, workspace_id) = fixture();
        let mut invalid = input("Bad");
        invalid.columns = vec!["secret".into()];
        assert!(invalid.validate().is_err());

        let other_id = "ws-other";
        let view = db
            .with_conn(|conn| create(conn, &workspace_id, &input("Mine")))
            .expect("create");
        assert!(db
            .with_conn(|conn| update(conn, other_id, &view.id, &input("Stolen")))
            .is_err());
        assert!(db
            .with_conn(|conn| delete(conn, other_id, &view.id))
            .is_err());
        assert!(db
            .with_conn(|conn| list(conn, other_id))
            .expect("other list")
            .is_empty());
    }
}
