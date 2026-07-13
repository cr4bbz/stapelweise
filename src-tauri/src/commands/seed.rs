use crate::db::DbState;
use crate::db::models::Deck;
use crate::seed::SeedGenerator;
use super::CommandError;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub fn seed_sample_data(state: State<DbState>) -> Result<Vec<Deck>, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let decks = SeedGenerator::generate(&db.repo)?;
    Ok(decks)
}
