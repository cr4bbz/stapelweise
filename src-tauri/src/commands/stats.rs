use crate::db::models::{DashboardStats, DeckStats};
use crate::db::DbState;
use super::CommandError;
use chrono::Local;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub fn get_deck_stats(state: State<DbState>, deck_id: String) -> Result<DeckStats, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let today_start = Local::now().format("%Y-%m-%d").to_string();
    let stats = db.repo.get_deck_stats(&deck_id, &today_start)?;
    Ok(stats)
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_dashboard_stats(state: State<DbState>) -> Result<DashboardStats, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let today = Local::now().format("%Y-%m-%d").to_string();
    let stats = db.repo.get_dashboard_stats(&today)?;
    Ok(stats)
}
