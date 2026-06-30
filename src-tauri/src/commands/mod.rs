pub mod decks;
pub mod cards;
pub mod seed;
pub mod settings;
pub mod stats;

use std::fmt;

#[derive(Debug)]
pub struct CommandError(pub String);

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<rusqlite::Error> for CommandError {
    fn from(e: rusqlite::Error) -> Self {
        CommandError(format!("Database error: {}", e))
    }
}

impl From<String> for CommandError {
    fn from(s: String) -> Self {
        CommandError(s)
    }
}

// Boilerplate to make CommandError work with Tauri's invoke error handling
impl serde::Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}
