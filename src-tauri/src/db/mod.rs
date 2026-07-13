pub mod migrations;
pub mod models;
pub mod repository;
pub mod settings;
pub mod backup;

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

use self::repository::Repository;
use self::settings::AppSettings;

pub struct Database {
    pub repo: Repository,
    settings: AppSettings,
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

        // Load or initialize settings (cached in memory)
        let settings = AppSettings::load(&repo).unwrap_or_else(|_| {
            let defaults = AppSettings::defaults();
            let _ = defaults.save(&repo);
            defaults
        });

        Ok(Self { repo, settings })
    }

    pub fn settings(&self) -> AppSettings {
        self.settings.clone()
    }

    /// Update settings in-memory and persist to DB.
    /// Returns the previous settings on success.
    pub fn update_settings(&mut self, settings: AppSettings) -> Result<(), rusqlite::Error> {
        settings.save(&self.repo)?;
        self.settings = settings;
        Ok(())
    }
}

// Thread-safe wrapper for Tauri commands
pub type DbState = Mutex<Database>;
