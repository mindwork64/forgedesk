use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;

use crate::domain::insights::{InsightBucket, InsightPoint, InsightsSummary};
use crate::error::CoreResult;

pub fn summary(
    conn: &Connection,
    workspace_id: &str,
    range_days: i64,
) -> CoreResult<InsightsSummary> {
    let range_days = range_days.clamp(7, 365);
    let now = Utc::now();
    let start = now - Duration::days(range_days);
    let start_text = start.to_rfc3339();
    let now_text = now.to_rfc3339();

    let count = |sql: &str, params: &[&dyn rusqlite::ToSql]| -> CoreResult<i64> {
        Ok(conn.query_row(sql, params, |row| row.get(0))?)
    };
    let total_created = count(
        "SELECT count(*) FROM issue WHERE workspace_id = ?1 AND deleted_at IS NULL AND created_at >= ?2",
        &[&workspace_id, &start_text],
    )?;
    let total_completed = count(
        "SELECT count(*) FROM issue WHERE workspace_id = ?1 AND deleted_at IS NULL AND completed_at >= ?2",
        &[&workspace_id, &start_text],
    )?;
    let open_count = count(
        "SELECT count(*) FROM issue WHERE workspace_id = ?1 AND deleted_at IS NULL AND completed_at IS NULL",
        &[&workspace_id],
    )?;
    let overdue_count = count(
        "SELECT count(*) FROM issue WHERE workspace_id = ?1 AND deleted_at IS NULL \
         AND completed_at IS NULL AND due_date IS NOT NULL AND due_date < ?2",
        &[&workspace_id, &now_text],
    )?;
    let avg_cycle_days: Option<f64> = conn.query_row(
        "SELECT avg((julianday(completed_at) - julianday(created_at))) FROM issue \
         WHERE workspace_id = ?1 AND deleted_at IS NULL AND completed_at >= ?2",
        rusqlite::params![workspace_id, start_text],
        |row| row.get(0),
    )?;

    let bucket_days = ((range_days as f64) / 12.0).ceil() as i64;
    let mut throughput = Vec::with_capacity(12);
    let mut buckets = vec![0i64; 12];
    {
        let mut statement = conn.prepare(
            "SELECT completed_at FROM issue WHERE workspace_id = ?1 AND deleted_at IS NULL \
             AND completed_at >= ?2",
        )?;
        let rows = statement.query_map(rusqlite::params![workspace_id, start_text], |row| {
            row.get::<_, String>(0)
        })?;
        for row in rows {
            let value = row?;
            if let Ok(completed) = DateTime::parse_from_rfc3339(&value) {
                let days = (completed.with_timezone(&Utc) - start).num_days();
                let index = (days / bucket_days).clamp(0, 11) as usize;
                buckets[index] += 1;
            }
        }
    }
    for (index, count) in buckets.into_iter().enumerate() {
        let bucket_start = start + Duration::days(index as i64 * bucket_days);
        throughput.push(InsightPoint {
            label: bucket_start.format("%b %d").to_string(),
            count,
        });
    }

    let by_status = buckets_for(
        conn,
        "SELECT s.name, count(*) FROM issue i JOIN workflow_status s ON s.id = i.status_id \
         WHERE i.workspace_id = ?1 AND i.deleted_at IS NULL GROUP BY s.id, s.name \
         ORDER BY count(*) DESC, s.position ASC",
        workspace_id,
    )?;
    let by_project = buckets_for(
        conn,
        "SELECT coalesce(p.name, 'No project'), count(*) FROM issue i \
         LEFT JOIN project p ON p.id = i.project_id \
         WHERE i.workspace_id = ?1 AND i.deleted_at IS NULL GROUP BY i.project_id \
         ORDER BY count(*) DESC LIMIT 8",
        workspace_id,
    )?;

    Ok(InsightsSummary {
        generated_at: now_text.clone(),
        data_through: now_text,
        range_days,
        total_created,
        total_completed,
        open_count,
        overdue_count,
        avg_cycle_days,
        throughput,
        by_status,
        by_project,
    })
}

fn buckets_for(conn: &Connection, sql: &str, workspace_id: &str) -> CoreResult<Vec<InsightBucket>> {
    let mut statement = conn.prepare(sql)?;
    let rows = statement.query_map([workspace_id], |row| {
        Ok(InsightBucket {
            label: row.get(0)?,
            count: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::domain::issue::{CreateIssueInput, IssueType, Priority};
    use crate::domain::time::now_rfc3339;

    #[test]
    fn summary_uses_local_aggregates() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("db");
        let workspace = db
            .with_conn(|conn| crate::db::repositories::workspace::create_initial(conn, "Insights"))
            .expect("workspace");
        let issue = db
            .with_conn(|conn| {
                crate::db::repositories::issues::create(
                    conn,
                    &workspace.id,
                    &CreateIssueInput {
                        title: "Completed issue".into(),
                        description: String::new(),
                        status_id: None,
                        priority: Priority::Medium,
                        issue_type: IssueType::Task,
                        estimate: None,
                        start_date: None,
                        due_date: None,
                        project_id: None,
                        parent_id: None,
                        label_names: vec![],
                    },
                )
            })
            .expect("issue");
        db.with_conn(|conn| {
            Ok(conn.execute(
                "UPDATE issue SET completed_at = ?1 WHERE id = ?2",
                rusqlite::params![now_rfc3339(), issue],
            )?)
        })
        .expect("complete");

        let result = db
            .with_conn(|conn| summary(conn, &workspace.id, 30))
            .expect("summary");
        assert_eq!(result.range_days, 30);
        assert!(result.total_created >= 1);
        assert!(result.total_completed >= 1);
        assert_eq!(
            result
                .throughput
                .iter()
                .map(|point| point.count)
                .sum::<i64>(),
            1
        );
        assert!(!result.by_status.is_empty());
        assert!(!result.by_project.is_empty());
    }
}
