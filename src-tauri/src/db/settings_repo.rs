use rusqlite::Connection;

use crate::error::AppError;
use crate::models::NotificationSettings;

pub fn get_notification_settings(conn: &Connection) -> Result<NotificationSettings, AppError> {
    let enabled: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            ["notification_enabled"],
            |row| row.get(0),
        )
        .ok();

    let time: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            ["notification_time"],
            |row| row.get(0),
        )
        .ok();

    let default = NotificationSettings::default();
    Ok(NotificationSettings {
        enabled: enabled
            .map(|v| v == "true")
            .unwrap_or(default.enabled),
        time: time.unwrap_or(default.time),
    })
}

pub fn save_notification_settings(
    conn: &Connection,
    settings: &NotificationSettings,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params!["notification_enabled", settings.enabled.to_string()],
    )?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params!["notification_time", &settings.time],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn setup_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        Database::new(conn).unwrap()
    }

    #[test]
    fn returns_default_when_no_settings_exist() {
        let db = setup_db();
        let conn = db.conn.lock().unwrap();
        let settings = get_notification_settings(&conn).unwrap();
        assert!(!settings.enabled);
        assert_eq!(settings.time, "10:00");
    }

    #[test]
    fn saves_and_loads_settings() {
        let db = setup_db();
        let conn = db.conn.lock().unwrap();

        let settings = NotificationSettings {
            enabled: true,
            time: "08:30".to_string(),
        };
        save_notification_settings(&conn, &settings).unwrap();

        let loaded = get_notification_settings(&conn).unwrap();
        assert!(loaded.enabled);
        assert_eq!(loaded.time, "08:30");
    }

    #[test]
    fn overwrites_existing_settings() {
        let db = setup_db();
        let conn = db.conn.lock().unwrap();

        let settings1 = NotificationSettings {
            enabled: true,
            time: "08:30".to_string(),
        };
        save_notification_settings(&conn, &settings1).unwrap();

        let settings2 = NotificationSettings {
            enabled: false,
            time: "22:00".to_string(),
        };
        save_notification_settings(&conn, &settings2).unwrap();

        let loaded = get_notification_settings(&conn).unwrap();
        assert!(!loaded.enabled);
        assert_eq!(loaded.time, "22:00");
    }
}
