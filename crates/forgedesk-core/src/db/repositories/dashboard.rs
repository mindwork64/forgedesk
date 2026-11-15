use rusqlite::{Connection, OptionalExtension, Row};

use crate::domain::dashboard::{Dashboard, DashboardWidget};
use crate::domain::time::now_rfc3339;
use crate::error::{CoreError, CoreResult};

const MAX_WIDGETS: i64 = 24;

pub fn get_or_create(conn: &Connection, workspace_id: &str) -> CoreResult<Dashboard> {
    if let Some(dashboard) = existing(conn, workspace_id)? {
        return Ok(dashboard);
    }
    let id = uuid::Uuid::now_v7().to_string();
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO dashboard (id, workspace_id, name, created_at, updated_at) \
         VALUES (?1, ?2, 'Overview', ?3, ?3)",
        rusqlite::params![id, workspace_id, now],
    )?;
    for (position, (kind, width)) in [("metric_open", 3), ("issues_recent", 6), ("api_budget", 3)]
        .into_iter()
        .enumerate()
    {
        insert_widget(conn, &id, kind, width, position as i64)?;
    }
    get(conn, workspace_id)
}

pub fn add_widget(
    conn: &Connection,
    workspace_id: &str,
    kind: &str,
    width: i64,
) -> CoreResult<Dashboard> {
    let dashboard = get_or_create(conn, workspace_id)?;
    if dashboard.widgets.len() as i64 >= MAX_WIDGETS {
        return Err(CoreError::validation(
            "widgets",
            "Dashboard already has the maximum number of widgets.",
        ));
    }
    validate_width(width)?;
    let position = dashboard.widgets.len() as i64;
    insert_widget(conn, &dashboard.id, kind, width, position)?;
    touch(conn, &dashboard.id)?;
    get(conn, workspace_id)
}

pub fn remove_widget(
    conn: &Connection,
    workspace_id: &str,
    widget_id: &str,
) -> CoreResult<Dashboard> {
    let dashboard = get_or_create(conn, workspace_id)?;
    let changed = conn.execute(
        "DELETE FROM dashboard_widget WHERE id = ?1 AND dashboard_id = ?2",
        rusqlite::params![widget_id, dashboard.id],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Dashboard widget"));
    }
    touch(conn, &dashboard.id)?;
    get(conn, workspace_id)
}

pub fn update_width(
    conn: &Connection,
    workspace_id: &str,
    widget_id: &str,
    width: i64,
) -> CoreResult<Dashboard> {
    validate_width(width)?;
    let dashboard = get_or_create(conn, workspace_id)?;
    let changed = conn.execute(
        "UPDATE dashboard_widget SET width = ?1, updated_at = ?2 \
         WHERE id = ?3 AND dashboard_id = ?4",
        rusqlite::params![width, now_rfc3339(), widget_id, dashboard.id],
    )?;
    if changed == 0 {
        return Err(CoreError::not_found("Dashboard widget"));
    }
    touch(conn, &dashboard.id)?;
    get(conn, workspace_id)
}

pub fn reorder(conn: &Connection, workspace_id: &str, ids: &[String]) -> CoreResult<Dashboard> {
    let dashboard = get_or_create(conn, workspace_id)?;
    let existing = dashboard
        .widgets
        .iter()
        .map(|widget| widget.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let requested = ids
        .iter()
        .map(String::as_str)
        .collect::<std::collections::HashSet<_>>();
    if ids.len() != dashboard.widgets.len() || existing != requested {
        return Err(CoreError::validation(
            "ids",
            "Reorder must include every dashboard widget exactly once.",
        ));
    }
    for (position, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE dashboard_widget SET position = ?1, updated_at = ?2 \
             WHERE id = ?3 AND dashboard_id = ?4",
            rusqlite::params![position as i64, now_rfc3339(), id, dashboard.id],
        )?;
    }
    touch(conn, &dashboard.id)?;
    get(conn, workspace_id)
}

fn existing(conn: &Connection, workspace_id: &str) -> CoreResult<Option<Dashboard>> {
    let id: Option<String> = conn
        .query_row(
            "SELECT id FROM dashboard WHERE workspace_id = ?1",
            [workspace_id],
            |row| row.get(0),
        )
        .optional()?;
    match id {
        Some(_) => get(conn, workspace_id).map(Some),
        None => Ok(None),
    }
}

fn get(conn: &Connection, workspace_id: &str) -> CoreResult<Dashboard> {
    let (id, name, updated_at): (String, String, String) = conn
        .query_row(
            "SELECT id, name, updated_at FROM dashboard WHERE workspace_id = ?1",
            [workspace_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| CoreError::not_found("Dashboard"))?;
    let mut statement = conn.prepare(
        "SELECT id, dashboard_id, kind, title, width, position FROM dashboard_widget \
         WHERE dashboard_id = ?1 ORDER BY position ASC, created_at ASC",
    )?;
    let rows = statement.query_map([&id], read_widget)?;
    Ok(Dashboard {
        id,
        name,
        widgets: rows.collect::<rusqlite::Result<Vec<_>>>()?,
        updated_at,
    })
}

fn insert_widget(
    conn: &Connection,
    dashboard_id: &str,
    kind: &str,
    width: i64,
    position: i64,
) -> CoreResult<()> {
    validate_width(width)?;
    let title = title_for(kind)?;
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO dashboard_widget \
         (id, dashboard_id, kind, title, width, position, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        rusqlite::params![
            uuid::Uuid::now_v7().to_string(),
            dashboard_id,
            kind,
            title,
            width,
            position,
            now
        ],
    )?;
    Ok(())
}

fn touch(conn: &Connection, dashboard_id: &str) -> CoreResult<()> {
    conn.execute(
        "UPDATE dashboard SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now_rfc3339(), dashboard_id],
    )?;
    Ok(())
}

fn title_for(kind: &str) -> CoreResult<&'static str> {
    match kind {
        "metric_open" => Ok("Open issues"),
        "metric_completed" => Ok("Completed"),
        "metric_overdue" => Ok("Overdue"),
        "issues_recent" => Ok("Recent issues"),
        "api_budget" => Ok("API budget"),
        _ => Err(CoreError::validation("kind", "Unsupported widget kind.")),
    }
}

fn validate_width(width: i64) -> CoreResult<()> {
    if matches!(width, 3 | 4 | 6 | 8 | 12) {
        Ok(())
    } else {
        Err(CoreError::validation(
            "width",
            "Widget width must be 3, 4, 6, 8 or 12 columns.",
        ))
    }
}

fn read_widget(row: &Row<'_>) -> rusqlite::Result<DashboardWidget> {
    Ok(DashboardWidget {
        id: row.get(0)?,
        dashboard_id: row.get(1)?,
        kind: row.get(2)?,
        title: row.get(3)?,
        width: row.get(4)?,
        position: row.get(5)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn fixture() -> (tempfile::TempDir, Database, String) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("db");
        let workspace = db
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "Dashboard"))
            .expect("workspace");
        (dir, db, workspace.id)
    }

    #[test]
    fn default_dashboard_is_created_once() {
        let (_dir, db, workspace_id) = fixture();
        let first = db
            .with_conn(|conn| get_or_create(conn, &workspace_id))
            .expect("first");
        let second = db
            .with_conn(|conn| get_or_create(conn, &workspace_id))
            .expect("second");
        assert_eq!(first.id, second.id);
        assert_eq!(second.widgets.len(), 3);
    }

    #[test]
    fn widgets_add_reorder_resize_remove() {
        let (_dir, db, workspace_id) = fixture();
        let added = db
            .with_conn(|conn| add_widget(conn, &workspace_id, "metric_completed", 4))
            .expect("add");
        assert_eq!(added.widgets.len(), 4);
        let ids = added
            .widgets
            .iter()
            .map(|widget| widget.id.clone())
            .rev()
            .collect::<Vec<_>>();
        let reordered = db
            .with_conn(|conn| reorder(conn, &workspace_id, &ids))
            .expect("reorder");
        assert_eq!(reordered.widgets[0].id, ids[0]);
        let resized = db
            .with_conn(|conn| update_width(conn, &workspace_id, &ids[0], 8))
            .expect("resize");
        assert_eq!(resized.widgets[0].width, 8);
        let removed = db
            .with_conn(|conn| remove_widget(conn, &workspace_id, &ids[0]))
            .expect("remove");
        assert_eq!(removed.widgets.len(), 3);
        assert!(db
            .with_conn(|conn| add_widget(conn, &workspace_id, "unknown", 3))
            .is_err());
        assert!(db
            .with_conn(|conn| update_width(conn, &workspace_id, &removed.widgets[0].id, 5))
            .is_err());
    }
}
