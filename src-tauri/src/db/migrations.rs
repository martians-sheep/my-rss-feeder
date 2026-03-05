use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::AppError;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(conn: Connection) -> Result<Self, AppError> {
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS feeds (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL,
                url TEXT NOT NULL UNIQUE,
                feed_type TEXT,
                site_url TEXT,
                description TEXT,
                icon_url TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_fetched_at TEXT,
                etag TEXT,
                last_modified TEXT
            );

            CREATE TABLE IF NOT EXISTS articles (
                id TEXT PRIMARY KEY NOT NULL,
                feed_id TEXT NOT NULL,
                entry_id TEXT NOT NULL,
                title TEXT NOT NULL,
                url TEXT,
                summary TEXT,
                content TEXT,
                author TEXT,
                published_at TEXT,
                is_read INTEGER NOT NULL DEFAULT 0,
                read_at TEXT,
                og_image_url TEXT,
                og_image_local TEXT,
                og_description TEXT,
                og_fetched INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                categories TEXT,
                FOREIGN KEY (feed_id) REFERENCES feeds(id) ON DELETE CASCADE,
                UNIQUE(feed_id, entry_id)
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_articles_published_at ON articles(published_at);
            CREATE INDEX IF NOT EXISTS idx_articles_created_at ON articles(created_at);",
        )?;

        // 既存DBへのマイグレーション: feed_type カラムを追加
        let _ = conn.execute_batch(
            "ALTER TABLE feeds ADD COLUMN feed_type TEXT;
             ALTER TABLE articles ADD COLUMN categories TEXT;",
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_creates_tables_successfully() {
        let conn = Connection::open_in_memory().unwrap();
        let db = Database::new(conn).unwrap();
        let conn = db.conn.lock().unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='feeds'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='articles'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='settings'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn migration_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        let db = Database::new(conn).unwrap();
        // Running migrations again should not fail
        db.run_migrations().unwrap();
    }

    #[test]
    fn foreign_keys_are_enabled() {
        let conn = Connection::open_in_memory().unwrap();
        let db = Database::new(conn).unwrap();
        let conn = db.conn.lock().unwrap();
        let fk_enabled: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk_enabled, 1);
    }
}
