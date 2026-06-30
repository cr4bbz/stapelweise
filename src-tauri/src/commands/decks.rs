use crate::db::DbState;
use crate::db::models::Deck;
use super::CommandError;
use tauri::State;

#[tauri::command]
pub fn create_deck(state: State<DbState>, name: String) -> Result<Deck, CommandError> {
    let db = state.lock().map_err(|e| CommandError(format!("Lock error: {}", e)))?;
    let deck = db.repo.create_deck(&name)?;
    Ok(deck)
}

#[tauri::command]
pub fn list_decks(state: State<DbState>) -> Result<Vec<Deck>, CommandError> {
    let db = state.lock().map_err(|e| CommandError(format!("Lock error: {}", e)))?;
    let decks = db.repo.list_decks()?;
    Ok(decks)
}

#[tauri::command]
pub fn get_deck(state: State<DbState>, deck_id: String) -> Result<Option<Deck>, CommandError> {
    let db = state.lock().map_err(|e| CommandError(format!("Lock error: {}", e)))?;
    let deck = db.repo.get_deck(&deck_id)?;
    Ok(deck)
}

#[tauri::command]
pub fn update_deck(state: State<DbState>, deck_id: String, name: String) -> Result<(), CommandError> {
    let db = state.lock().map_err(|e| CommandError(format!("Lock error: {}", e)))?;
    db.repo.update_deck(&deck_id, &name)?;
    Ok(())
}

#[tauri::command]
pub fn delete_deck(state: State<DbState>, deck_id: String) -> Result<(), CommandError> {
    let db = state.lock().map_err(|e| CommandError(format!("Lock error: {}", e)))?;
    db.repo.delete_deck(&deck_id)?;
    Ok(())
}
