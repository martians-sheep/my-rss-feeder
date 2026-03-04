use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::Feed;

pub fn insert_feed(conn: &Connection, feed: &Feed) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO feeds (id, title, url, feed_type, site_url, description, icon_url, created_at, updated_at, last_fetched_at, etag, last_modified)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            feed.id,
            feed.title,
            feed.url,
            feed.feed_type,
            feed.site_url,
            feed.description,
            feed.icon_url,
            feed.created_at,
            feed.updated_at,
            feed.last_fetched_at,
            feed.etag,
            feed.last_modified,
        ],
    )?;
    Ok(())
}

pub fn get_feed_by_id(conn: &Connection, id: &str) -> Result<Feed, AppError> {
    conn.query_row(
        "SELECT id, title, url, feed_type, site_url, description, icon_url, created_at, updated_at, last_fetched_at, etag, last_modified
         FROM feeds WHERE id = ?1",
        params![id],
        row_to_feed,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("Feed not found: {}", id))
        }
        _ => AppError::Database(e),
    })
}

pub fn get_feed_by_url(conn: &Connection, url: &str) -> Result<Option<Feed>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, title, url, feed_type, site_url, description, icon_url, created_at, updated_at, last_fetched_at, etag, last_modified
         FROM feeds WHERE url = ?1",
    )?;
    let mut rows = stmt.query(params![url])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_feed(row)?)),
        None => Ok(None),
    }
}

pub fn list_feeds(conn: &Connection) -> Result<Vec<Feed>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, title, url, feed_type, site_url, description, icon_url, created_at, updated_at, last_fetched_at, etag, last_modified
         FROM feeds ORDER BY title ASC",
    )?;
    let feeds = stmt
        .query_map([], row_to_feed)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(feeds)
}

pub fn update_feed(conn: &Connection, feed: &Feed) -> Result<(), AppError> {
    conn.execute(
        "UPDATE feeds SET title = ?1, url = ?2, feed_type = ?3, site_url = ?4, description = ?5, icon_url = ?6,
         updated_at = ?7, last_fetched_at = ?8, etag = ?9, last_modified = ?10
         WHERE id = ?11",
        params![
            feed.title,
            feed.url,
            feed.feed_type,
            feed.site_url,
            feed.description,
            feed.icon_url,
            feed.updated_at,
            feed.last_fetched_at,
            feed.etag,
            feed.last_modified,
            feed.id,
        ],
    )?;
    Ok(())
}

pub fn delete_feed(conn: &Connection, id: &str) -> Result<(), AppError> {
    let affected = conn.execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("Feed not found: {}", id)));
    }
    Ok(())
}

fn row_to_feed(row: &rusqlite::Row) -> rusqlite::Result<Feed> {
    Ok(Feed {
        id: row.get(0)?,
        title: row.get(1)?,
        url: row.get(2)?,
        feed_type: row.get(3)?,
        site_url: row.get(4)?,
        description: row.get(5)?,
        icon_url: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        last_fetched_at: row.get(9)?,
        etag: row.get(10)?,
        last_modified: row.get(11)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn setup() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        Database::new(conn).unwrap()
    }

    fn sample_feed() -> Feed {
        Feed {
            id: uuid::Uuid::new_v4().to_string(),
            title: "Test Feed".to_string(),
            url: "https://example.com/feed.xml".to_string(),
            feed_type: Some("rss2".to_string()),
            site_url: Some("https://example.com".to_string()),
            description: Some("A test feed".to_string()),
            icon_url: None,
            created_at: "2024-01-01T00:00:00+00:00".to_string(),
            updated_at: "2024-01-01T00:00:00+00:00".to_string(),
            last_fetched_at: None,
            etag: None,
            last_modified: None,
        }
    }

    #[test]
    fn insert_and_get_feed() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        insert_feed(&conn, &feed).unwrap();
        let retrieved = get_feed_by_id(&conn, &feed.id).unwrap();
        assert_eq!(retrieved.title, "Test Feed");
        assert_eq!(retrieved.url, "https://example.com/feed.xml");
    }

    #[test]
    fn list_feeds_returns_all_ordered_by_title() {
        let db = setup();
        let conn = db.conn.lock().unwrap();

        let mut feed1 = sample_feed();
        feed1.title = "BBB Feed".to_string();
        feed1.url = "https://example.com/feed1.xml".to_string();

        let mut feed2 = sample_feed();
        feed2.title = "AAA Feed".to_string();
        feed2.url = "https://example.com/feed2.xml".to_string();

        insert_feed(&conn, &feed1).unwrap();
        insert_feed(&conn, &feed2).unwrap();

        let feeds = list_feeds(&conn).unwrap();
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds[0].title, "AAA Feed");
        assert_eq!(feeds[1].title, "BBB Feed");
    }

    #[test]
    fn duplicate_url_returns_error() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        insert_feed(&conn, &feed).unwrap();

        let mut dup = sample_feed();
        dup.id = uuid::Uuid::new_v4().to_string();
        let result = insert_feed(&conn, &dup);
        assert!(result.is_err());
    }

    #[test]
    fn delete_feed_removes_it() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        insert_feed(&conn, &feed).unwrap();
        delete_feed(&conn, &feed.id).unwrap();

        let result = get_feed_by_id(&conn, &feed.id);
        assert!(result.is_err());
    }

    #[test]
    fn delete_nonexistent_feed_returns_not_found() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let result = delete_feed(&conn, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn update_feed_changes_fields() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let mut feed = sample_feed();
        insert_feed(&conn, &feed).unwrap();

        feed.title = "Updated Title".to_string();
        feed.updated_at = "2024-02-01T00:00:00+00:00".to_string();
        update_feed(&conn, &feed).unwrap();

        let retrieved = get_feed_by_id(&conn, &feed.id).unwrap();
        assert_eq!(retrieved.title, "Updated Title");
    }

    #[test]
    fn get_feed_by_url_returns_matching_feed() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        insert_feed(&conn, &feed).unwrap();

        let found = get_feed_by_url(&conn, &feed.url).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, feed.id);
    }

    #[test]
    fn get_feed_by_url_returns_none_for_unknown() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let found = get_feed_by_url(&conn, "https://unknown.com/feed").unwrap();
        assert!(found.is_none());
    }
}
