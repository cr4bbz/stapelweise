use crate::db::DbState;
use crate::db::models::Deck;
use super::CommandError;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub fn create_deck(state: State<DbState>, name: String) -> Result<Deck, CommandError> {
    let db = state.lock().map_err(|e| CommandError(format!("Lock error: {}", e)))?;
    let deck = db.repo.create_deck(&name)?;
    Ok(deck)
}

#[tauri::command(rename_all = "camelCase")]
pub fn list_decks(state: State<DbState>) -> Result<Vec<Deck>, CommandError> {
    let db = state.lock().map_err(|e| CommandError(format!("Lock error: {}", e)))?;
    let decks = db.repo.list_decks()?;
    Ok(decks)
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_deck(state: State<DbState>, deck_id: String) -> Result<Option<Deck>, CommandError> {
    let db = state.lock().map_err(|e| CommandError(format!("Lock error: {}", e)))?;
    let deck = db.repo.get_deck(&deck_id)?;
    Ok(deck)
}

#[tauri::command(rename_all = "camelCase")]
pub fn update_deck(state: State<DbState>, deck_id: String, name: String) -> Result<(), CommandError> {
    let db = state.lock().map_err(|e| CommandError(format!("Lock error: {}", e)))?;
    db.repo.update_deck(&deck_id, &name)?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn delete_deck(state: State<DbState>, deck_id: String) -> Result<(), CommandError> {
    let db = state.lock().map_err(|e| CommandError(format!("Lock error: {}", e)))?;
    db.repo.delete_deck(&deck_id)?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct JsonCard {
    pub front: String,
    pub back: String,
    pub reasoning: Option<String>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub card_type: String,
}

#[tauri::command(rename_all = "camelCase")]
pub fn import_deck(state: State<DbState>, name: String, cards: Vec<JsonCard>) -> Result<(), CommandError> {
    let db = state.lock().map_err(|e| CommandError(format!("Lock error: {}", e)))?;
    
    let deck = db.repo.create_deck(&name)?;
    for c in cards {
        let card_type = if c.card_type.is_empty() { "basic" } else { &c.card_type };
        db.repo.create_card(
            &deck.id, 
            card_type, 
            None,
            c.reasoning.as_deref(), 
            &c.front, 
            &c.back, 
            c.tags
        )?;
    }
    
    Ok(())
}
