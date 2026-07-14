use super::CommandError;
use crate::db::models::{Card, CardState, Review, SearchResult};
use crate::db::DbState;
use crate::srs::sm2;
use chrono::{Local, NaiveDate, Utc};
use tauri::State;
use uuid::Uuid;

fn normalize_language(value: Option<String>) -> Option<String> {
    value
        .map(|language| language.trim().replace('_', "-").to_lowercase())
        .filter(|language| !language.is_empty())
}

// ── Card CRUD ────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn create_card(
    state: State<DbState>,
    deck_id: String,
    card_type: Option<String>,
    content: Option<String>,
    reasoning: Option<String>,
    front: String,
    back: String,
    tags: Option<Vec<String>>,
    front_language: Option<String>,
    back_language: Option<String>,
) -> Result<Card, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let c_type = card_type.unwrap_or_else(|| "basic".to_string());
    let tags_vec = tags.unwrap_or_default();
    let front_language = normalize_language(front_language);
    let back_language = normalize_language(back_language);
    let mut card = db.repo.create_card(
        &deck_id,
        &c_type,
        content.as_deref(),
        reasoning.as_deref(),
        &front,
        &back,
        tags_vec,
    )?;
    db.repo.update_card_languages(
        &card.id,
        front_language.as_deref(),
        back_language.as_deref(),
    )?;
    card.front_language = front_language;
    card.back_language = back_language;
    Ok(card)
}

#[tauri::command(rename_all = "camelCase")]
pub fn list_cards(state: State<DbState>, deck_id: String) -> Result<Vec<Card>, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let cards = db.repo.list_cards(&deck_id)?;
    Ok(cards)
}

#[tauri::command(rename_all = "camelCase")]
pub fn update_card(
    state: State<DbState>,
    card_id: String,
    card_type: Option<String>,
    content: Option<String>,
    reasoning: Option<String>,
    front: String,
    back: String,
    tags: Option<Vec<String>>,
    front_language: Option<String>,
    back_language: Option<String>,
) -> Result<(), CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let c_type = card_type.unwrap_or_else(|| "basic".to_string());
    let tags_vec = tags.unwrap_or_default();
    db.repo.update_card(
        &card_id,
        &c_type,
        content.as_deref(),
        reasoning.as_deref(),
        &front,
        &back,
        tags_vec,
    )?;
    let front_language = normalize_language(front_language);
    let back_language = normalize_language(back_language);
    db.repo.update_card_languages(
        &card_id,
        front_language.as_deref(),
        back_language.as_deref(),
    )?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn delete_card(state: State<DbState>, card_id: String) -> Result<(), CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    db.repo.delete_card(&card_id)?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_card_state(
    state: State<DbState>,
    card_id: String,
) -> Result<Option<CardState>, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let state = db.repo.get_card_state(&card_id)?;
    Ok(state)
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_all_tags(state: State<DbState>) -> Result<Vec<String>, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let tags = db.repo.get_all_tags()?;
    Ok(tags)
}

// ── Study ────────────────────────────────────────────

/// Return type for due cards: the card data plus its SM-2 state
#[derive(serde::Serialize)]
pub struct DueCard {
    pub card: Card,
    pub state: CardState,
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_due_cards(
    state: State<DbState>,
    deck_ids: Vec<String>,
    limit: u32,
) -> Result<Vec<DueCard>, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let settings = db.settings();
    let effective_limit = limit.min(settings.session_limit);
    let cards = db.repo.get_due_cards(&deck_ids, effective_limit)?;
    Ok(cards
        .into_iter()
        .map(|(card, state)| DueCard { card, state })
        .collect())
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_due_cards_by_tags(
    state: State<DbState>,
    tags: Vec<String>,
    limit: u32,
) -> Result<Vec<DueCard>, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let settings = db.settings();
    let effective_limit = limit.min(settings.session_limit);
    let cards = db.repo.get_due_cards_by_tags(&tags, effective_limit)?;
    Ok(cards
        .into_iter()
        .map(|(card, state)| DueCard { card, state })
        .collect())
}

#[tauri::command(rename_all = "camelCase")]
pub fn count_due_cards(state: State<DbState>, deck_id: String) -> Result<u32, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let count = db.repo.count_due_cards(&deck_id)?;
    Ok(count)
}

#[tauri::command(rename_all = "camelCase")]
pub fn count_total_cards(state: State<DbState>, deck_id: String) -> Result<u32, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let count = db.repo.count_total_cards(&deck_id)?;
    Ok(count)
}

#[tauri::command(rename_all = "camelCase")]
pub fn submit_review(
    state: State<DbState>,
    card_id: String,
    quality: u8,
) -> Result<CardState, CommandError> {
    if quality > 5 {
        return Err(CommandError::validation("quality must be 0-5"));
    }

    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let now = Utc::now();
    let today = Local::now().date_naive();

    // Load user settings for SM-2 config
    let settings = db.settings();
    let sm2_config = settings.to_sm2_config();

    // Get current SM-2 state, convert to sm2::Sm2State
    let current_state = db
        .repo
        .get_card_state(&card_id)?
        .ok_or_else(|| CommandError::not_found(&format!("Card {} not found", card_id)))?;

    let sm2_input = sm2::Sm2State {
        interval: current_state.interval,
        ease_factor: current_state.ease_factor,
        repetitions: current_state.repetitions,
        next_review: NaiveDate::parse_from_str(&current_state.next_review, "%Y-%m-%d")
            .unwrap_or(today),
    };

    // Advance via SM-2 with user config
    let next =
        sm2::sm2_advance(&sm2_input, quality, today, &sm2_config).map_err(CommandError::from)?;

    let now_str = now.format("%Y-%m-%dT%H:%M:%S").to_string();

    // Save previous state for undo
    let prev_state_json = serde_json::to_string(&current_state).ok();

    // Create audit log entry
    let review = Review {
        id: Uuid::new_v4().to_string(),
        card_id: card_id.clone(),
        quality,
        reviewed_at: now_str.clone(),
        interval: next.interval,
        ease_factor: next.ease_factor,
        repetitions: next.repetitions,
        prev_state: prev_state_json,
    };
    let new_streak = if quality >= sm2_config.pass_threshold {
        current_state.correct_streak + 1
    } else {
        0
    };

    let updated_state = CardState {
        card_id: card_id.clone(),
        interval: next.interval,
        ease_factor: next.ease_factor,
        repetitions: next.repetitions,
        next_review: next.next_review.format("%Y-%m-%d").to_string(),
        total_reviews: current_state.total_reviews + 1,
        correct_streak: new_streak,
        last_review: Some(now_str),
    };

    db.repo.apply_review(&review, &updated_state)?;

    Ok(updated_state)
}

#[tauri::command(rename_all = "camelCase")]
pub fn undo_last_review(
    state: State<DbState>,
    deck_id: String,
) -> Result<Option<DueCard>, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let restored = db.repo.undo_last_review(&deck_id)?;
    Ok(restored.map(|(card, state)| DueCard { card, state }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn search_cards(
    state: State<DbState>,
    query: String,
) -> Result<Vec<SearchResult>, CommandError> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let results = db.repo.search_cards(&query)?;
    Ok(results)
}
