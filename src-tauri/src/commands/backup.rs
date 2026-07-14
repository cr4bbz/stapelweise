use crate::commands::CommandError;
use crate::db::backup::{export_backup, inspect_backup, restore_backup, ImportInspection};
use crate::db::DbState;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub fn export_backup_data(state: State<DbState>) -> Result<String, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let pkg = export_backup(&db.repo)?;
    let json_str = serde_json::to_string_pretty(&pkg).map_err(|e| {
        CommandError::export_failed("Fehler beim Erstellen des Backup-JSON", Some(e.to_string()))
    })?;
    Ok(json_str)
}

#[tauri::command(rename_all = "camelCase")]
pub fn inspect_backup_data(
    state: State<DbState>,
    json_data: String,
) -> Result<ImportInspection, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let inspection = inspect_backup(&db.repo, &json_data)?;
    Ok(inspection)
}

#[tauri::command(rename_all = "camelCase")]
pub fn restore_backup_data(
    state: State<DbState>,
    json_data: String,
    conflict_strategy: String,
) -> Result<(), CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    restore_backup(&db.repo, &json_data, &conflict_strategy)?;
    Ok(())
}
