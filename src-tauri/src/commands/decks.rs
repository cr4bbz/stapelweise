use super::CommandError;
use crate::db::models::Deck;
use crate::db::DbState;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub fn create_deck(state: State<DbState>, name: String) -> Result<Deck, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let deck = db.repo.create_deck(&name)?;
    Ok(deck)
}

#[tauri::command(rename_all = "camelCase")]
pub fn list_decks(
    state: State<DbState>,
    include_archived: Option<bool>,
) -> Result<Vec<Deck>, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let decks = if include_archived.unwrap_or(false) {
        db.repo.list_all_decks()?
    } else {
        db.repo.list_decks()?
    };
    Ok(decks)
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_deck(state: State<DbState>, deck_id: String) -> Result<Option<Deck>, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let deck = db.repo.get_deck(&deck_id)?;
    Ok(deck)
}

#[tauri::command(rename_all = "camelCase")]
pub fn update_deck(
    state: State<DbState>,
    deck_id: String,
    name: String,
) -> Result<(), CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    db.repo.update_deck(&deck_id, &name)?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn delete_deck(state: State<DbState>, deck_id: String) -> Result<(), CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    db.repo.delete_deck(&deck_id)?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn archive_deck(state: State<DbState>, deck_id: String) -> Result<(), CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    db.repo.set_deck_archived(&deck_id, true)?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn restore_deck(state: State<DbState>, deck_id: String) -> Result<(), CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    db.repo.set_deck_archived(&deck_id, false)?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct JsonCard {
    pub front: String,
    pub back: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub front_language: Option<String>,
    #[serde(default)]
    pub back_language: Option<String>,
    #[serde(default)]
    pub reasoning: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub card_type: String,
}

#[tauri::command(rename_all = "camelCase")]
pub fn import_deck(
    state: State<DbState>,
    name: String,
    cards: Vec<JsonCard>,
) -> Result<(), CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;

    let deck = db.repo.create_deck(&name)?;
    for c in cards {
        let card_type = if c.card_type.is_empty() {
            "basic"
        } else {
            &c.card_type
        };
        let card = db.repo.create_card(
            &deck.id,
            card_type,
            c.content.as_deref(),
            c.reasoning.as_deref(),
            &c.front,
            &c.back,
            c.tags,
        )?;
        let front_language = c
            .front_language
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let back_language = c
            .back_language
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        db.repo
            .update_card_languages(&card.id, front_language, back_language)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::JsonCard;

    #[test]
    fn json_card_preserves_structured_content() {
        let card: JsonCard = serde_json::from_str(
            r#"{
                "front": "Frage",
                "back": "[x] A\n[ ] B",
                "card_type": "multiple_choice",
                "content": "{\"options\":[{\"text\":\"A\",\"correct\":true},{\"text\":\"B\",\"correct\":false}]}"
            }"#,
        )
        .unwrap();

        assert_eq!(card.card_type, "multiple_choice");
        assert!(card
            .content
            .as_deref()
            .is_some_and(|content| content.contains("options")));
        assert!(card.tags.is_empty());
    }
}
