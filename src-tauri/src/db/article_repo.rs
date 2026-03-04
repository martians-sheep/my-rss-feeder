use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::Article;

pub fn upsert_article(conn: &Connection, article: &Article) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO articles (id, feed_id, entry_id, title, url, summary, content, author, published_at, is_read, read_at, og_image_url, og_image_local, og_description, og_fetched, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
         ON CONFLICT(feed_id, entry_id) DO UPDATE SET
           title = excluded.title,
           url = excluded.url,
           summary = excluded.summary,
           content = excluded.content,
           author = excluded.author,
           published_at = excluded.published_at",
        params![
            article.id,
            article.feed_id,
            article.entry_id,
            article.title,
            article.url,
            article.summary,
            article.content,
            article.author,
            article.published_at,
            article.is_read as i32,
            article.read_at,
            article.og_image_url,
            article.og_image_local,
            article.og_description,
            article.og_fetched as i32,
            article.created_at,
        ],
    )?;
    Ok(())
}

pub fn list_articles(
    conn: &Connection,
    feed_id: Option<&str>,
    date_from: Option<&str>,
    date_to: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Article>, AppError> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut param_idx = 0usize;

    if let Some(fid) = feed_id {
        param_idx += 1;
        conditions.push(format!("a.feed_id = ?{}", param_idx));
        params_vec.push(Box::new(fid.to_string()));
    }
    if let Some(df) = date_from {
        param_idx += 1;
        conditions.push(format!(
            "COALESCE(a.published_at, a.created_at) >= ?{}",
            param_idx
        ));
        params_vec.push(Box::new(df.to_string()));
    }
    if let Some(dt) = date_to {
        param_idx += 1;
        conditions.push(format!(
            "COALESCE(a.published_at, a.created_at) < ?{}",
            param_idx
        ));
        params_vec.push(Box::new(dt.to_string()));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    param_idx += 1;
    let limit_idx = param_idx;
    params_vec.push(Box::new(limit));
    param_idx += 1;
    let offset_idx = param_idx;
    params_vec.push(Box::new(offset));

    let sql = format!(
        "SELECT a.id, a.feed_id, a.entry_id, a.title, a.url, a.summary, a.content, a.author,
                a.published_at, a.is_read, a.read_at, a.og_image_url, a.og_image_local,
                a.og_description, a.og_fetched, a.created_at, f.title as feed_title
         FROM articles a LEFT JOIN feeds f ON a.feed_id = f.id
         {}
         ORDER BY a.published_at DESC, a.created_at DESC
         LIMIT ?{} OFFSET ?{}",
        where_clause, limit_idx, offset_idx
    );

    let mut stmt = conn.prepare(&sql)?;
    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        params_vec.iter().map(|p| p.as_ref()).collect();
    let articles = stmt
        .query_map(params_refs.as_slice(), row_to_article)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(articles)
}

pub fn get_article_by_id(conn: &Connection, id: &str) -> Result<Article, AppError> {
    conn.query_row(
        "SELECT a.id, a.feed_id, a.entry_id, a.title, a.url, a.summary, a.content, a.author,
                a.published_at, a.is_read, a.read_at, a.og_image_url, a.og_image_local,
                a.og_description, a.og_fetched, a.created_at, f.title as feed_title
         FROM articles a LEFT JOIN feeds f ON a.feed_id = f.id
         WHERE a.id = ?1",
        params![id],
        row_to_article,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("Article not found: {}", id))
        }
        _ => AppError::Database(e),
    })
}

pub fn mark_as_read(conn: &Connection, id: &str) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let affected = conn.execute(
        "UPDATE articles SET is_read = 1, read_at = ?1 WHERE id = ?2",
        params![now, id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("Article not found: {}", id)));
    }
    Ok(())
}

pub fn update_ogp(
    conn: &Connection,
    article_id: &str,
    og_image_url: Option<&str>,
    og_image_local: Option<&str>,
    og_description: Option<&str>,
) -> Result<(), AppError> {
    let affected = conn.execute(
        "UPDATE articles SET og_image_url = ?1, og_image_local = ?2, og_description = ?3, og_fetched = 1 WHERE id = ?4",
        params![og_image_url, og_image_local, og_description, article_id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!(
            "Article not found: {}",
            article_id
        )));
    }
    Ok(())
}

pub fn list_unfetched_ogp_articles(
    conn: &Connection,
    article_ids: &[String],
) -> Result<Vec<Article>, AppError> {
    if article_ids.is_empty() {
        return Ok(vec![]);
    }

    let placeholders: Vec<String> = (1..=article_ids.len()).map(|i| format!("?{}", i)).collect();
    let sql = format!(
        "SELECT a.id, a.feed_id, a.entry_id, a.title, a.url, a.summary, a.content, a.author,
                a.published_at, a.is_read, a.read_at, a.og_image_url, a.og_image_local,
                a.og_description, a.og_fetched, a.created_at, f.title as feed_title
         FROM articles a LEFT JOIN feeds f ON a.feed_id = f.id
         WHERE a.id IN ({}) AND a.og_fetched = 0",
        placeholders.join(", ")
    );

    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::types::ToSql> = article_ids
        .iter()
        .map(|id| id as &dyn rusqlite::types::ToSql)
        .collect();
    let articles = stmt
        .query_map(params.as_slice(), row_to_article)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(articles)
}

fn row_to_article(row: &rusqlite::Row) -> rusqlite::Result<Article> {
    Ok(Article {
        id: row.get(0)?,
        feed_id: row.get(1)?,
        entry_id: row.get(2)?,
        title: row.get(3)?,
        url: row.get(4)?,
        summary: row.get(5)?,
        content: row.get(6)?,
        author: row.get(7)?,
        published_at: row.get(8)?,
        is_read: row.get::<_, i32>(9)? != 0,
        read_at: row.get(10)?,
        og_image_url: row.get(11)?,
        og_image_local: row.get(12)?,
        og_description: row.get(13)?,
        og_fetched: row.get::<_, i32>(14)? != 0,
        created_at: row.get(15)?,
        feed_title: row.get(16)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::feed_repo;
    use crate::db::Database;
    use crate::models::Feed;

    fn setup() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        Database::new(conn).unwrap()
    }

    fn sample_feed() -> Feed {
        Feed {
            id: "feed-1".to_string(),
            title: "Test Feed".to_string(),
            url: "https://example.com/feed.xml".to_string(),
            site_url: Some("https://example.com".to_string()),
            description: None,
            icon_url: None,
            created_at: "2024-01-01T00:00:00+00:00".to_string(),
            updated_at: "2024-01-01T00:00:00+00:00".to_string(),
            last_fetched_at: None,
            etag: None,
            last_modified: None,
        }
    }

    fn sample_article(feed_id: &str, entry_id: &str) -> Article {
        Article {
            id: uuid::Uuid::new_v4().to_string(),
            feed_id: feed_id.to_string(),
            entry_id: entry_id.to_string(),
            title: format!("Article {}", entry_id),
            url: Some(format!("https://example.com/{}", entry_id)),
            summary: None,
            content: None,
            author: None,
            published_at: Some("2024-01-01T00:00:00+00:00".to_string()),
            is_read: false,
            read_at: None,
            og_image_url: None,
            og_image_local: None,
            og_description: None,
            og_fetched: false,
            created_at: "2024-01-01T00:00:00+00:00".to_string(),
            feed_title: None,
        }
    }

    #[test]
    fn upsert_and_get_article() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        feed_repo::insert_feed(&conn, &feed).unwrap();

        let article = sample_article(&feed.id, "entry-1");
        upsert_article(&conn, &article).unwrap();

        let retrieved = get_article_by_id(&conn, &article.id).unwrap();
        assert_eq!(retrieved.title, "Article entry-1");
        assert_eq!(retrieved.feed_title, Some("Test Feed".to_string()));
    }

    #[test]
    fn upsert_updates_existing_article() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        feed_repo::insert_feed(&conn, &feed).unwrap();

        let article = sample_article(&feed.id, "entry-1");
        upsert_article(&conn, &article).unwrap();

        let mut updated = article.clone();
        updated.id = uuid::Uuid::new_v4().to_string();
        updated.title = "Updated Title".to_string();
        upsert_article(&conn, &updated).unwrap();

        let articles = list_articles(&conn, Some(&feed.id), None, None, 100, 0).unwrap();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].title, "Updated Title");
    }

    #[test]
    fn list_articles_filters_by_feed_id() {
        let db = setup();
        let conn = db.conn.lock().unwrap();

        let feed1 = sample_feed();
        feed_repo::insert_feed(&conn, &feed1).unwrap();

        let mut feed2 = sample_feed();
        feed2.id = "feed-2".to_string();
        feed2.url = "https://example.com/feed2.xml".to_string();
        feed_repo::insert_feed(&conn, &feed2).unwrap();

        let a1 = sample_article("feed-1", "e1");
        let a2 = sample_article("feed-2", "e2");
        upsert_article(&conn, &a1).unwrap();
        upsert_article(&conn, &a2).unwrap();

        let articles = list_articles(&conn, Some("feed-1"), None, None, 100, 0).unwrap();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].feed_id, "feed-1");

        let all = list_articles(&conn, None, None, None, 100, 0).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn mark_article_as_read() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        feed_repo::insert_feed(&conn, &feed).unwrap();

        let article = sample_article(&feed.id, "entry-1");
        upsert_article(&conn, &article).unwrap();

        mark_as_read(&conn, &article.id).unwrap();

        let retrieved = get_article_by_id(&conn, &article.id).unwrap();
        assert!(retrieved.is_read);
        assert!(retrieved.read_at.is_some());
    }

    #[test]
    fn mark_nonexistent_article_returns_not_found() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let result = mark_as_read(&conn, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn cascade_delete_removes_articles() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        feed_repo::insert_feed(&conn, &feed).unwrap();

        let article = sample_article(&feed.id, "entry-1");
        upsert_article(&conn, &article).unwrap();

        feed_repo::delete_feed(&conn, &feed.id).unwrap();

        let articles = list_articles(&conn, Some(&feed.id), None, None, 100, 0).unwrap();
        assert_eq!(articles.len(), 0);
    }

    #[test]
    fn update_ogp_sets_fields_and_marks_fetched() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        feed_repo::insert_feed(&conn, &feed).unwrap();

        let article = sample_article(&feed.id, "entry-ogp");
        upsert_article(&conn, &article).unwrap();

        update_ogp(
            &conn,
            &article.id,
            Some("https://example.com/og.png"),
            Some("/cache/abc.png"),
            Some("OG description"),
        )
        .unwrap();

        let retrieved = get_article_by_id(&conn, &article.id).unwrap();
        assert!(retrieved.og_fetched);
        assert_eq!(
            retrieved.og_image_url.as_deref(),
            Some("https://example.com/og.png")
        );
        assert_eq!(retrieved.og_image_local.as_deref(), Some("/cache/abc.png"));
        assert_eq!(
            retrieved.og_description.as_deref(),
            Some("OG description")
        );
    }

    #[test]
    fn update_ogp_with_none_values() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        feed_repo::insert_feed(&conn, &feed).unwrap();

        let article = sample_article(&feed.id, "entry-ogp-none");
        upsert_article(&conn, &article).unwrap();

        update_ogp(&conn, &article.id, None, None, None).unwrap();

        let retrieved = get_article_by_id(&conn, &article.id).unwrap();
        assert!(retrieved.og_fetched);
        assert!(retrieved.og_image_url.is_none());
    }

    #[test]
    fn update_ogp_nonexistent_article_returns_not_found() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let result = update_ogp(&conn, "nonexistent", None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn list_unfetched_ogp_articles_filters_correctly() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        feed_repo::insert_feed(&conn, &feed).unwrap();

        let a1 = sample_article(&feed.id, "e-fetched");
        let a2 = sample_article(&feed.id, "e-unfetched");
        upsert_article(&conn, &a1).unwrap();
        upsert_article(&conn, &a2).unwrap();

        // Mark a1 as OGP-fetched
        update_ogp(&conn, &a1.id, None, None, None).unwrap();

        let ids = vec![a1.id.clone(), a2.id.clone()];
        let unfetched = list_unfetched_ogp_articles(&conn, &ids).unwrap();
        assert_eq!(unfetched.len(), 1);
        assert_eq!(unfetched[0].id, a2.id);
    }

    #[test]
    fn list_unfetched_ogp_articles_empty_input() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let result = list_unfetched_ogp_articles(&conn, &[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn list_articles_respects_limit_and_offset() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        feed_repo::insert_feed(&conn, &feed).unwrap();

        for i in 0..5 {
            let mut a = sample_article(&feed.id, &format!("entry-{}", i));
            a.published_at = Some(format!("2024-01-0{}T00:00:00+00:00", i + 1));
            upsert_article(&conn, &a).unwrap();
        }

        let page1 = list_articles(&conn, None, None, None, 2, 0).unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = list_articles(&conn, None, None, None, 2, 2).unwrap();
        assert_eq!(page2.len(), 2);

        let page3 = list_articles(&conn, None, None, None, 2, 4).unwrap();
        assert_eq!(page3.len(), 1);
    }

    #[test]
    fn list_articles_filters_by_date_from() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        feed_repo::insert_feed(&conn, &feed).unwrap();

        let mut a1 = sample_article(&feed.id, "old");
        a1.published_at = Some("2024-01-01T00:00:00+00:00".to_string());
        let mut a2 = sample_article(&feed.id, "new");
        a2.published_at = Some("2024-06-01T00:00:00+00:00".to_string());
        upsert_article(&conn, &a1).unwrap();
        upsert_article(&conn, &a2).unwrap();

        let articles = list_articles(
            &conn,
            None,
            Some("2024-03-01T00:00:00+00:00"),
            None,
            100,
            0,
        )
        .unwrap();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].entry_id, "new");
    }

    #[test]
    fn list_articles_filters_by_date_to() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        feed_repo::insert_feed(&conn, &feed).unwrap();

        let mut a1 = sample_article(&feed.id, "old");
        a1.published_at = Some("2024-01-01T00:00:00+00:00".to_string());
        let mut a2 = sample_article(&feed.id, "new");
        a2.published_at = Some("2024-06-01T00:00:00+00:00".to_string());
        upsert_article(&conn, &a1).unwrap();
        upsert_article(&conn, &a2).unwrap();

        let articles = list_articles(
            &conn,
            None,
            None,
            Some("2024-03-01T00:00:00+00:00"),
            100,
            0,
        )
        .unwrap();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].entry_id, "old");
    }

    #[test]
    fn list_articles_filters_by_date_range() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        feed_repo::insert_feed(&conn, &feed).unwrap();

        let mut a1 = sample_article(&feed.id, "jan");
        a1.published_at = Some("2024-01-15T00:00:00+00:00".to_string());
        let mut a2 = sample_article(&feed.id, "mar");
        a2.published_at = Some("2024-03-15T00:00:00+00:00".to_string());
        let mut a3 = sample_article(&feed.id, "jun");
        a3.published_at = Some("2024-06-15T00:00:00+00:00".to_string());
        upsert_article(&conn, &a1).unwrap();
        upsert_article(&conn, &a2).unwrap();
        upsert_article(&conn, &a3).unwrap();

        let articles = list_articles(
            &conn,
            None,
            Some("2024-02-01T00:00:00+00:00"),
            Some("2024-05-01T00:00:00+00:00"),
            100,
            0,
        )
        .unwrap();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].entry_id, "mar");
    }

    #[test]
    fn list_articles_null_published_at_falls_back() {
        let db = setup();
        let conn = db.conn.lock().unwrap();
        let feed = sample_feed();
        feed_repo::insert_feed(&conn, &feed).unwrap();

        let mut a1 = sample_article(&feed.id, "no-pub");
        a1.published_at = None;
        a1.created_at = "2024-04-01T00:00:00+00:00".to_string();
        upsert_article(&conn, &a1).unwrap();

        // Should match using created_at fallback
        let articles = list_articles(
            &conn,
            None,
            Some("2024-03-01T00:00:00+00:00"),
            Some("2024-05-01T00:00:00+00:00"),
            100,
            0,
        )
        .unwrap();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].entry_id, "no-pub");

        // Should not match when outside range
        let articles = list_articles(
            &conn,
            None,
            Some("2024-05-01T00:00:00+00:00"),
            None,
            100,
            0,
        )
        .unwrap();
        assert_eq!(articles.len(), 0);
    }

    #[test]
    fn list_articles_date_and_feed_id_combined() {
        let db = setup();
        let conn = db.conn.lock().unwrap();

        let feed1 = sample_feed();
        feed_repo::insert_feed(&conn, &feed1).unwrap();

        let mut feed2 = sample_feed();
        feed2.id = "feed-2".to_string();
        feed2.url = "https://example.com/feed2.xml".to_string();
        feed_repo::insert_feed(&conn, &feed2).unwrap();

        let mut a1 = sample_article("feed-1", "f1-old");
        a1.published_at = Some("2024-01-01T00:00:00+00:00".to_string());
        let mut a2 = sample_article("feed-1", "f1-new");
        a2.published_at = Some("2024-06-01T00:00:00+00:00".to_string());
        let mut a3 = sample_article("feed-2", "f2-new");
        a3.published_at = Some("2024-06-01T00:00:00+00:00".to_string());
        upsert_article(&conn, &a1).unwrap();
        upsert_article(&conn, &a2).unwrap();
        upsert_article(&conn, &a3).unwrap();

        // feed-1 + date_from: should only get f1-new
        let articles = list_articles(
            &conn,
            Some("feed-1"),
            Some("2024-03-01T00:00:00+00:00"),
            None,
            100,
            0,
        )
        .unwrap();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].entry_id, "f1-new");
    }
}
