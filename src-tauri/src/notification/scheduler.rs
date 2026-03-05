use std::sync::Arc;

use chrono::{Local, NaiveTime};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::watch;

use crate::db::Database;
use crate::models::NotificationSettings;

/// Sender half for broadcasting setting changes to the scheduler.
pub type SettingsChangedSender = Arc<watch::Sender<NotificationSettings>>;

/// Parse a "HH:MM" string into (hour, minute).
pub fn parse_time(time_str: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let hour = parts[0].parse::<u32>().ok()?;
    let minute = parts[1].parse::<u32>().ok()?;
    if hour >= 24 || minute >= 60 {
        return None;
    }
    Some((hour, minute))
}

/// Calculate the duration from now until the next occurrence of the target time.
fn duration_until(hour: u32, minute: u32) -> std::time::Duration {
    let now = Local::now();
    let today_target = now
        .date_naive()
        .and_time(NaiveTime::from_hms_opt(hour, minute, 0).unwrap());

    let target = if now.naive_local() < today_target {
        today_target
    } else {
        // Already past today's target time, schedule for tomorrow
        today_target + chrono::Duration::days(1)
    };

    let diff = target - now.naive_local();
    diff.to_std().unwrap_or(std::time::Duration::from_secs(60))
}

/// Count unread articles created today.
fn count_todays_unread(db: &Database) -> i64 {
    let conn = db.conn.lock().unwrap();
    let today = Local::now().format("%Y-%m-%d").to_string();
    let pattern = format!("{}%", today);
    conn.query_row(
        "SELECT COUNT(*) FROM articles WHERE is_read = 0 AND created_at LIKE ?1",
        [&pattern],
        |row| row.get(0),
    )
    .unwrap_or(0)
}

/// Start the notification scheduler background task.
pub fn start(
    app_handle: tauri::AppHandle,
    db: Arc<Database>,
    mut settings_rx: watch::Receiver<NotificationSettings>,
) {
    tauri::async_runtime::spawn(async move {
        loop {
            let settings = settings_rx.borrow().clone();

            if !settings.enabled {
                // Notifications disabled — wait for settings to change
                if settings_rx.changed().await.is_err() {
                    break; // Channel closed, shut down
                }
                continue;
            }

            let (hour, minute) = match parse_time(&settings.time) {
                Some(t) => t,
                None => {
                    // Invalid time — wait for settings to change
                    if settings_rx.changed().await.is_err() {
                        break;
                    }
                    continue;
                }
            };

            let wait = duration_until(hour, minute);

            tokio::select! {
                _ = tokio::time::sleep(wait) => {
                    // Time reached — check for unread articles
                    let unread = count_todays_unread(&db);
                    if unread > 0 {
                        let _ = app_handle.notification()
                            .builder()
                            .title("My RSS Feeder")
                            .body(format!("本日の未読記事が{}件あります", unread))
                            .show();
                    }
                }
                result = settings_rx.changed() => {
                    if result.is_err() {
                        break; // Channel closed
                    }
                    // Settings changed — restart the loop with new settings
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_time_valid() {
        assert_eq!(parse_time("10:00"), Some((10, 0)));
        assert_eq!(parse_time("00:00"), Some((0, 0)));
        assert_eq!(parse_time("23:59"), Some((23, 59)));
        assert_eq!(parse_time("08:30"), Some((8, 30)));
    }

    #[test]
    fn parse_time_invalid() {
        assert_eq!(parse_time("24:00"), None);
        assert_eq!(parse_time("10:60"), None);
        assert_eq!(parse_time("abc"), None);
        assert_eq!(parse_time("10"), None);
        assert_eq!(parse_time(""), None);
        assert_eq!(parse_time("10:00:00"), None);
    }
}
