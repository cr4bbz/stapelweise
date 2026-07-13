pub mod decks;
pub mod cards;
pub mod exams;
pub mod test_engine;
pub mod seed;
pub mod settings;
pub mod stats;
pub mod obsidian;

use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Serialize)]
pub struct CommandErrorPayload {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub recoverable: bool,
}

#[derive(Debug, Clone)]
pub struct CommandError(pub CommandErrorPayload);

impl CommandError {
    pub fn new(code: &str, message: &str, details: Option<String>, recoverable: bool) -> Self {
        Self(CommandErrorPayload {
            code: code.to_string(),
            message: message.to_string(),
            details,
            recoverable,
        })
    }

    pub fn validation(msg: &str) -> Self {
        Self::new("VALIDATION_FAILED", msg, None, true)
    }

    pub fn not_found(msg: &str) -> Self {
        Self::new("NOT_FOUND", msg, None, true)
    }

    pub fn import_failed(msg: &str, details: Option<String>) -> Self {
        Self::new("IMPORT_FAILED", msg, details, true)
    }

    pub fn export_failed(msg: &str, details: Option<String>) -> Self {
        Self::new("EXPORT_FAILED", msg, details, true)
    }

    pub fn obsidian_sync(msg: &str, details: Option<String>) -> Self {
        Self::new("OBSIDIAN_SYNC_ERROR", msg, details, true)
    }

    pub fn internal(msg: &str) -> Self {
        Self::new("INTERNAL_ERROR", msg, None, false)
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]: {}", self.0.code, self.0.message)
    }
}

impl From<rusqlite::Error> for CommandError {
    fn from(e: rusqlite::Error) -> Self {
        CommandError::new("DB_ERROR", "Datenbankfehler aufgetreten", Some(e.to_string()), true)
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(e: serde_json::Error) -> Self {
        CommandError::new("IMPORT_INVALID_JSON", "Ungültiges JSON-Format", Some(e.to_string()), true)
    }
}

impl From<String> for CommandError {
    fn from(s: String) -> Self {
        if s.starts_with("VALIDATION:") {
            CommandError::validation(s.trim_start_matches("VALIDATION:"))
        } else if s.contains("not found") || s.contains("nicht gefunden") {
            CommandError::not_found(&s)
        } else {
            CommandError::internal(&s)
        }
    }
}

impl From<&str> for CommandError {
    fn from(s: &str) -> Self {
        CommandError::from(s.to_string())
    }
}

// Boilerplate to make CommandError serialize as a structured error payload for Tauri IPC
impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.0.serialize(serializer)
    }
}
