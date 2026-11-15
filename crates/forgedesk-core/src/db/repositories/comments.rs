use rusqlite::Connection;

use crate::domain::time::now_rfc3339;
use crate::domain::{validate_comment, Comment, CommentAuthor};
use crate::error::{CoreError, CoreResult};

pub fn add(conn: &Connection, issue_id: &str, body: &str) -> CoreResult<Comment> {
    let body = validate_comment(body)?;
    ensure_issue_exists(conn, issue_id)?;

    let now = now_rfc3339();
    let comment = Comment {
        id: uuid::Uuid::now_v7().to_string(),
        issue_id: issue_id.to_owned(),
        body,
        author: CommentAuthor::Local,
        created_at: now.clone(),
        updated_at: now,
    };
    conn.execute(
        "INSERT INTO comment (id, issue_id, body, author, created_at, updated_at) \
         VALUES (?1, ?2, ?3, 'local', ?4, ?4)",
        rusqlite::params![
            comment.id,
            comment.issue_id,
            comment.body,
            comment.created_at
        ],
    )?;
    Ok(comment)
}

pub fn list(conn: &Connection, issue_id: &str) -> CoreResult<Vec<Comment>> {
    let mut statement = conn.prepare(
        "SELECT id, issue_id, body, author, created_at, updated_at FROM comment \
         WHERE issue_id = ?1 AND deleted_at IS NULL ORDER BY created_at ASC, rowid ASC",
    )?;
    let rows = statement.query_map([issue_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
        ))
    })?;

    let mut comments = Vec::new();
    for row in rows {
        let (id, issue_id, body, author, created_at, updated_at) = row?;
        comments.push(Comment {
            id,
            issue_id,
            body,
            author: match author.as_str() {
                "remote" => CommentAuthor::Remote,
                _ => CommentAuthor::Local,
            },
            created_at,
            updated_at,
        });
    }
    Ok(comments)
}

fn ensure_issue_exists(conn: &Connection, issue_id: &str) -> CoreResult<()> {
    let count: i64 = conn.query_row(
        "SELECT count(*) FROM issue WHERE id = ?1 AND deleted_at IS NULL",
        [issue_id],
        |row| row.get(0),
    )?;
    if count == 0 {
        return Err(CoreError::not_found("Issue"));
    }
    Ok(())
}
