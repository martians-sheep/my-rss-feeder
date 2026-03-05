use std::path::{Path, PathBuf};

use tauri::{AppHandle, State};

use crate::db::article_repo;
use crate::db::feed_repo;
use crate::db::settings_repo;
use crate::db::Database;
use crate::error::AppError;
use crate::feed::{discovery, fetcher, parser};
use crate::models::{Article, ArticleSortOrder, Feed, NotificationSettings};
use crate::notification::scheduler::SettingsChangedSender;
use crate::ogp::{self, OgpResult, OgpResultData};
use crate::webview;

#[tauri::command]
pub async fn add_feed(
    url: String,
    db: State<'_, std::sync::Arc<Database>>,
    client: State<'_, reqwest::Client>,
) -> Result<Feed, AppError> {
    // HTMLページの場合はフィードURLを自動検出する
    let feed_url = discovery::discover_or_use_feed_url(&client, &url).await?;

    // 重複チェック
    {
        let conn = db.conn.lock().unwrap();
        if let Some(existing) = feed_repo::get_feed_by_url(&conn, &feed_url)? {
            return Err(AppError::Duplicate(format!(
                "Feed already exists: {}",
                existing.url
            )));
        }
    }

    // フィードを取得
    let fetch_result = fetcher::fetch_feed(&client, &feed_url, None, None)
        .await?
        .ok_or_else(|| AppError::Other("No content returned from feed".to_string()))?;

    // フィードをパース
    let parsed = parser::parse_feed(&fetch_result.body, &feed_url)?;

    let now = chrono::Utc::now().to_rfc3339();
    let feed_id = uuid::Uuid::new_v4().to_string();

    let feed = Feed {
        id: feed_id.clone(),
        title: parsed.title,
        url: feed_url.clone(),
        feed_type: Some(parsed.feed_type),
        site_url: parsed.site_url,
        description: parsed.description,
        icon_url: parsed.icon_url,
        created_at: now.clone(),
        updated_at: now.clone(),
        last_fetched_at: Some(now.clone()),
        etag: fetch_result.etag,
        last_modified: fetch_result.last_modified,
    };

    // フィードと記事を保存
    {
        let conn = db.conn.lock().unwrap();
        feed_repo::insert_feed(&conn, &feed)?;

        for entry in parsed.entries {
            let categories = if entry.categories.is_empty() {
                None
            } else {
                Some(entry.categories.join(","))
            };
            let article = Article {
                id: uuid::Uuid::new_v4().to_string(),
                feed_id: feed_id.clone(),
                entry_id: entry.entry_id,
                title: entry.title,
                url: entry.url,
                summary: entry.summary,
                content: entry.content,
                author: entry.author,
                published_at: entry.published_at,
                is_read: false,
                read_at: None,
                og_image_url: None,
                og_image_local: None,
                og_description: None,
                og_fetched: false,
                created_at: now.clone(),
                feed_title: None,
                categories,
                feed_order: entry.position as i32,
            };
            article_repo::upsert_article(&conn, &article)?;
        }
    }

    Ok(feed)
}

#[tauri::command]
pub fn list_feeds(db: State<'_, std::sync::Arc<Database>>) -> Result<Vec<Feed>, AppError> {
    let conn = db.conn.lock().unwrap();
    feed_repo::list_feeds(&conn)
}

#[tauri::command]
pub fn remove_feed(feed_id: String, db: State<'_, std::sync::Arc<Database>>) -> Result<(), AppError> {
    let conn = db.conn.lock().unwrap();
    feed_repo::delete_feed(&conn, &feed_id)
}

#[tauri::command]
pub async fn refresh_feed(
    feed_id: String,
    db: State<'_, std::sync::Arc<Database>>,
    client: State<'_, reqwest::Client>,
) -> Result<u32, AppError> {
    let feed = {
        let conn = db.conn.lock().unwrap();
        feed_repo::get_feed_by_id(&conn, &feed_id)?
    };

    let fetch_result = fetcher::fetch_feed(
        &client,
        &feed.url,
        feed.etag.as_deref(),
        feed.last_modified.as_deref(),
    )
    .await?;

    let fetch_result = match fetch_result {
        Some(r) => r,
        None => return Ok(0), // 304 Not Modified
    };

    let parsed = parser::parse_feed(&fetch_result.body, &feed.url)?;
    let now = chrono::Utc::now().to_rfc3339();
    let count = parsed.entries.len() as u32;

    {
        let conn = db.conn.lock().unwrap();

        let mut updated_feed = feed.clone();
        updated_feed.title = parsed.title;
        updated_feed.feed_type = Some(parsed.feed_type);
        updated_feed.site_url = parsed.site_url;
        updated_feed.description = parsed.description;
        updated_feed.icon_url = parsed.icon_url;
        updated_feed.updated_at = now.clone();
        updated_feed.last_fetched_at = Some(now.clone());
        updated_feed.etag = fetch_result.etag;
        updated_feed.last_modified = fetch_result.last_modified;
        feed_repo::update_feed(&conn, &updated_feed)?;

        for entry in parsed.entries {
            let categories = if entry.categories.is_empty() {
                None
            } else {
                Some(entry.categories.join(","))
            };
            let article = Article {
                id: uuid::Uuid::new_v4().to_string(),
                feed_id: feed_id.clone(),
                entry_id: entry.entry_id,
                title: entry.title,
                url: entry.url,
                summary: entry.summary,
                content: entry.content,
                author: entry.author,
                published_at: entry.published_at,
                is_read: false,
                read_at: None,
                og_image_url: None,
                og_image_local: None,
                og_description: None,
                og_fetched: false,
                created_at: now.clone(),
                feed_title: None,
                categories,
                feed_order: entry.position as i32,
            };
            article_repo::upsert_article(&conn, &article)?;
        }
    }

    Ok(count)
}

#[tauri::command]
pub async fn refresh_all_feeds(
    db: State<'_, std::sync::Arc<Database>>,
    client: State<'_, reqwest::Client>,
) -> Result<u32, AppError> {
    let feeds = {
        let conn = db.conn.lock().unwrap();
        feed_repo::list_feeds(&conn)?
    };

    let mut total = 0u32;

    for feed in feeds {
        let fetch_result = fetcher::fetch_feed(
            &client,
            &feed.url,
            feed.etag.as_deref(),
            feed.last_modified.as_deref(),
        )
        .await?;

        let fetch_result = match fetch_result {
            Some(r) => r,
            None => continue,
        };

        let parsed = parser::parse_feed(&fetch_result.body, &feed.url)?;
        let now = chrono::Utc::now().to_rfc3339();
        total += parsed.entries.len() as u32;

        {
            let conn = db.conn.lock().unwrap();

            let mut updated_feed = feed.clone();
            updated_feed.title = parsed.title;
            updated_feed.feed_type = Some(parsed.feed_type);
            updated_feed.site_url = parsed.site_url;
            updated_feed.description = parsed.description;
            updated_feed.icon_url = parsed.icon_url;
            updated_feed.updated_at = now.clone();
            updated_feed.last_fetched_at = Some(now.clone());
            updated_feed.etag = fetch_result.etag;
            updated_feed.last_modified = fetch_result.last_modified;
            feed_repo::update_feed(&conn, &updated_feed)?;

            for entry in parsed.entries {
                let categories = if entry.categories.is_empty() {
                    None
                } else {
                    Some(entry.categories.join(","))
                };
                let article = Article {
                    id: uuid::Uuid::new_v4().to_string(),
                    feed_id: feed.id.clone(),
                    entry_id: entry.entry_id,
                    title: entry.title,
                    url: entry.url,
                    summary: entry.summary,
                    content: entry.content,
                    author: entry.author,
                    published_at: entry.published_at,
                    is_read: false,
                    read_at: None,
                    og_image_url: None,
                    og_image_local: None,
                    og_description: None,
                    og_fetched: false,
                    created_at: now.clone(),
                    feed_title: None,
                    categories,
                    feed_order: entry.position as i32,
                };
                article_repo::upsert_article(&conn, &article)?;
            }
        }
    }

    Ok(total)
}

#[tauri::command]
pub fn list_articles(
    feed_id: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    sort_order: Option<ArticleSortOrder>,
    limit: Option<i64>,
    offset: Option<i64>,
    db: State<'_, std::sync::Arc<Database>>,
) -> Result<Vec<Article>, AppError> {
    let conn = db.conn.lock().unwrap();
    article_repo::list_articles(
        &conn,
        feed_id.as_deref(),
        date_from.as_deref(),
        date_to.as_deref(),
        sort_order.unwrap_or_default(),
        limit.unwrap_or(50),
        offset.unwrap_or(0),
    )
}

#[tauri::command]
pub fn mark_article_read(
    article_id: String,
    db: State<'_, std::sync::Arc<Database>>,
) -> Result<(), AppError> {
    let conn = db.conn.lock().unwrap();
    article_repo::mark_as_read(&conn, &article_id)
}

#[tauri::command]
pub async fn fetch_ogp_for_article(
    article_id: String,
    db: State<'_, std::sync::Arc<Database>>,
    client: State<'_, reqwest::Client>,
    app_handle: tauri::AppHandle,
) -> Result<OgpResult, AppError> {
    let article = {
        let conn = db.conn.lock().unwrap();
        article_repo::get_article_by_id(&conn, &article_id)?
    };

    let page_url = match &article.url {
        Some(url) => url.clone(),
        None => {
            return Ok(OgpResult {
                article_id,
                success: false,
                data: None,
                error: Some("Article has no URL".to_string()),
            });
        }
    };

    let cache_dir = get_image_cache_dir(&app_handle)?;
    let result = fetch_and_cache_ogp(&article_id, &page_url, &client, &cache_dir).await;

    // Update DB with result
    persist_ogp_result(&db, &result);

    Ok(result)
}

#[tauri::command]
pub async fn fetch_ogp_batch(
    article_ids: Vec<String>,
    db: State<'_, std::sync::Arc<Database>>,
    client: State<'_, reqwest::Client>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<OgpResult>, AppError> {
    let articles = {
        let conn = db.conn.lock().unwrap();
        article_repo::list_unfetched_ogp_articles(&conn, &article_ids)?
    };

    let cache_dir = get_image_cache_dir(&app_handle)?;
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(5));
    let mut handles = Vec::new();

    for article in articles {
        let sem = semaphore.clone();
        let client = client.inner().clone();
        let cache_dir = cache_dir.clone();
        let article_id = article.id.clone();
        let page_url = article.url.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            match page_url {
                Some(url) => fetch_and_cache_ogp(&article_id, &url, &client, &cache_dir).await,
                None => OgpResult {
                    article_id,
                    success: false,
                    data: None,
                    error: Some("Article has no URL".to_string()),
                },
            }
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => results.push(OgpResult {
                article_id: String::new(),
                success: false,
                data: None,
                error: Some(format!("Task join error: {}", e)),
            }),
        }
    }

    // Update DB with all results sequentially
    for result in &results {
        persist_ogp_result(&db, result);
    }

    Ok(results)
}

/// Fetch page HTML, extract OGP, and cache image. Pure HTTP work, no DB access.
async fn fetch_and_cache_ogp(
    article_id: &str,
    page_url: &str,
    client: &reqwest::Client,
    cache_dir: &Path,
) -> OgpResult {
    match do_fetch_ogp(page_url, client, cache_dir).await {
        Ok(data) => OgpResult {
            article_id: article_id.to_string(),
            success: true,
            data: Some(data),
            error: None,
        },
        Err(e) => OgpResult {
            article_id: article_id.to_string(),
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

async fn do_fetch_ogp(
    page_url: &str,
    client: &reqwest::Client,
    cache_dir: &Path,
) -> Result<OgpResultData, AppError> {
    let ogp_data = ogp::fetcher::fetch_ogp(client, page_url).await?;

    let og_image_local = match &ogp_data.og_image_url {
        Some(img_url) => ogp::image_cache::cache_image(client, img_url, cache_dir)
            .await?
            .map(|p| p.to_string_lossy().to_string()),
        None => None,
    };

    Ok(OgpResultData {
        og_image_url: ogp_data.og_image_url,
        og_image_local,
        og_description: ogp_data.og_description,
    })
}

/// Persist OGP result to DB. Marks as fetched even on error to avoid retrying broken URLs.
fn persist_ogp_result(db: &Database, result: &OgpResult) {
    if result.article_id.is_empty() {
        return;
    }
    let conn = db.conn.lock().unwrap();
    let _ = match &result.data {
        Some(data) => article_repo::update_ogp(
            &conn,
            &result.article_id,
            data.og_image_url.as_deref(),
            data.og_image_local.as_deref(),
            data.og_description.as_deref(),
        ),
        None => article_repo::update_ogp(&conn, &result.article_id, None, None, None),
    };
}

fn get_image_cache_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    use tauri::Manager;
    let app_dir = app_handle.path().app_data_dir().map_err(|e| {
        AppError::Other(format!("Failed to get app data dir: {}", e))
    })?;
    Ok(app_dir.join("image_cache"))
}

#[tauri::command]
pub fn get_notification_settings(
    db: State<'_, std::sync::Arc<Database>>,
) -> Result<NotificationSettings, AppError> {
    let conn = db.conn.lock().unwrap();
    settings_repo::get_notification_settings(&conn)
}

#[tauri::command]
pub fn save_notification_settings(
    settings: NotificationSettings,
    db: State<'_, std::sync::Arc<Database>>,
    sender: State<'_, SettingsChangedSender>,
) -> Result<(), AppError> {
    {
        let conn = db.conn.lock().unwrap();
        settings_repo::save_notification_settings(&conn, &settings)?;
    }
    let _ = sender.send(settings);
    Ok(())
}

#[tauri::command]
pub async fn open_article_webview(
    url: String,
    title: Option<String>,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    webview::open_article_webview(&app_handle, &url, title.as_deref())
}

#[tauri::command]
pub async fn highlight_in_webview(
    title: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    webview::highlight_in_webview(&app_handle, &title)
}

#[tauri::command]
pub async fn remove_highlight_in_webview(app_handle: AppHandle) -> Result<(), AppError> {
    webview::remove_highlight_in_webview(&app_handle)
}

#[tauri::command]
pub async fn close_article_webview(app_handle: AppHandle) -> Result<(), AppError> {
    webview::close_article_webview(&app_handle)
}

#[tauri::command]
pub async fn update_article_webview_bounds(
    left_offset: f64,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    webview::update_article_webview_bounds(&app_handle, left_offset)
}

#[tauri::command]
pub async fn hide_article_webview(app_handle: AppHandle) -> Result<(), AppError> {
    webview::hide_article_webview(&app_handle)
}

#[tauri::command]
pub async fn show_article_webview(app_handle: AppHandle) -> Result<(), AppError> {
    webview::show_article_webview(&app_handle)
}
