pub mod commands;
pub mod db;
pub mod error;
pub mod feed;
pub mod models;
pub mod notification;
pub mod ogp;
pub mod webview;

use std::sync::Arc;

use db::Database;
use rusqlite::Connection;
use std::path::PathBuf;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_dir: PathBuf = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join("feeds.db");
            let conn = Connection::open(&db_path)?;
            let database = Database::new(conn)?;
            let db = Arc::new(database);
            app.manage(db.clone());

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .user_agent("my-rss-feeder/0.1.0")
                .build()
                .expect("Failed to build HTTP client");
            app.manage(client);

            // Load initial notification settings and start scheduler
            let initial_settings = {
                let conn = db.conn.lock().unwrap();
                crate::db::settings_repo::get_notification_settings(&conn)
                    .unwrap_or_default()
            };
            let (settings_tx, settings_rx) =
                tokio::sync::watch::channel(initial_settings);
            let sender: notification::scheduler::SettingsChangedSender =
                Arc::new(settings_tx);
            app.manage(sender);

            notification::scheduler::start(app.handle().clone(), db, settings_rx);

            // Resize article webview when window is resized
            let app_handle = app.handle().clone();
            if let Some(window) = app.get_window("main") {
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Resized(_) = event {
                        let _ = webview::resize_article_webview(&app_handle);
                    }
                });
            }

            #[cfg(debug_assertions)]
            if let Some(window) = app.get_webview_window("main") {
                window.open_devtools();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_feed,
            commands::list_feeds,
            commands::remove_feed,
            commands::refresh_feed,
            commands::refresh_all_feeds,
            commands::list_articles,
            commands::mark_article_read,
            commands::fetch_ogp_for_article,
            commands::fetch_ogp_batch,
            commands::get_notification_settings,
            commands::save_notification_settings,
            commands::open_article_webview,
            commands::close_article_webview,
            commands::update_article_webview_bounds,
            commands::hide_article_webview,
            commands::show_article_webview,
            commands::highlight_in_webview,
            commands::remove_highlight_in_webview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
