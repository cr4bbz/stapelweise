use crate::db::DbState;
use crate::db::settings::AppSettings;
use super::CommandError;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub fn get_settings(state: State<DbState>) -> Result<AppSettings, CommandError> {
    let db = state.lock().map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    Ok(db.settings())
}

#[tauri::command(rename_all = "camelCase")]
pub fn update_settings(
    state: State<DbState>,
    settings: AppSettings,
) -> Result<(), CommandError> {
    let mut db = state.lock().map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    db.update_settings(settings)?;
    Ok(())
}
