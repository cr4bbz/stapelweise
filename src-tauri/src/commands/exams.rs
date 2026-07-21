use super::CommandError;
use crate::db::models::{Exam, ExamStats};
use crate::db::DbState;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub fn create_exam(
    state: State<DbState>,
    name: String,
    exam_type: String,
    exam_date: String,
    deck_ids: Vec<String>,
) -> Result<Exam, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let exam = db
        .repo
        .create_exam(&name, &exam_type, &exam_date, deck_ids)?;
    Ok(exam)
}

#[tauri::command(rename_all = "camelCase")]
pub fn list_exams(
    state: State<DbState>,
    include_archived: Option<bool>,
) -> Result<Vec<Exam>, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let exams = db.repo.list_exams(include_archived.unwrap_or(false))?;
    Ok(exams)
}

#[tauri::command(rename_all = "camelCase")]
pub fn archive_exam(state: State<DbState>, id: String) -> Result<(), CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    db.repo.set_exam_archived(&id, true)?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn restore_exam(state: State<DbState>, id: String) -> Result<(), CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    db.repo.set_exam_archived(&id, false)?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn delete_exam(state: State<DbState>, id: String) -> Result<(), CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    db.repo.delete_exam(&id)?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn update_exam(
    state: State<DbState>,
    id: String,
    name: String,
    exam_type: String,
    exam_date: String,
    deck_ids: Vec<String>,
) -> Result<Exam, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let exam = db
        .repo
        .update_exam(&id, &name, &exam_type, &exam_date, deck_ids)?;
    Ok(exam)
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_exam_stats(state: State<DbState>, id: String) -> Result<ExamStats, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let stats = db.repo.get_exam_stats(&id)?;
    Ok(stats)
}
