pub mod migrations;
pub mod models;
pub mod repository;

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

use self::repository::Repository;

pub struct Database {
    pub repo: Repository,
}

impl Database {
    pub fn new(app_data_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&app_data_dir)?;

        let db_path = app_data_dir.join("stapelweise.db");
        let conn = Connection::open(&db_path)?;

        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        migrations::run_migrations(&conn)?;

        let repo = Repository::new(conn);

        Ok(Self { repo })
    }
}

// Thread-safe wrapper for Tauri commands
pub type DbState = Mutex<Database>;
