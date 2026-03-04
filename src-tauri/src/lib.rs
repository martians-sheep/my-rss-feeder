pub mod commands;
pub mod db;
pub mod error;
pub mod feed;
pub mod models;
pub mod ogp;

use db::Database;
use rusqlite::Connection;
use std::path::PathBuf;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            let app_dir: PathBuf = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join("feeds.db");
            let conn = Connection::open(&db_path)?;
            let database = Database::new(conn)?;
            app.manage(database);

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .user_agent("my-rss-feeder/0.1.0")
                .build()
                .expect("Failed to build HTTP client");
            app.manage(client);

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
