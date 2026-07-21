use crate::commands::CommandError;
use crate::db::models::{ExamQuestion, ExamResult, ExamSession, ExamTemplate};
use crate::db::DbState;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub fn create_exam_template(
    state: State<DbState>,
    name: String,
    deck_ids: Vec<String>,
    tags: Vec<String>,
    allowed_types: Vec<String>,
    question_count: u32,
    time_limit_minutes: u32,
    pass_percentage: f64,
    seed: Option<u64>,
) -> Result<ExamTemplate, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let template = db.repo.create_exam_template(
        &name,
        deck_ids,
        tags,
        allowed_types,
        question_count,
        time_limit_minutes,
        pass_percentage,
        seed,
    )?;
    Ok(template)
}

#[tauri::command(rename_all = "camelCase")]
pub fn list_exam_templates(state: State<DbState>) -> Result<Vec<ExamTemplate>, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let templates = db.repo.list_exam_templates()?;
    Ok(templates)
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_exam_template(
    state: State<DbState>,
    id: String,
) -> Result<Option<ExamTemplate>, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let template = db.repo.get_exam_template(&id)?;
    Ok(template)
}

#[tauri::command(rename_all = "camelCase")]
pub fn delete_exam_template(state: State<DbState>, id: String) -> Result<(), CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    db.repo.delete_exam_template(&id)?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn start_exam_session(
    state: State<DbState>,
    template_id: Option<String>,
    name: String,
    deck_ids: Vec<String>,
    tags: Vec<String>,
    allowed_types: Vec<String>,
    question_count: u32,
    seed: Option<u64>,
) -> Result<(ExamSession, Vec<ExamQuestion>), CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let res = db.repo.start_exam_session(
        template_id,
        &name,
        deck_ids,
        tags,
        allowed_types,
        question_count,
        seed,
    )?;
    Ok(res)
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_exam_session_with_questions(
    state: State<DbState>,
    session_id: String,
) -> Result<Option<(ExamSession, Vec<ExamQuestion>)>, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let res = db.repo.get_exam_session_with_questions(&session_id)?;
    Ok(res)
}

#[tauri::command(rename_all = "camelCase")]
pub fn submit_exam_question_answer(
    state: State<DbState>,
    question_id: String,
    user_answer: String,
) -> Result<ExamQuestion, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let question = db
        .repo
        .submit_exam_question_answer(&question_id, &user_answer)?;
    Ok(question)
}

#[tauri::command(rename_all = "camelCase")]
pub fn finish_exam_session(
    state: State<DbState>,
    session_id: String,
    pass_percentage: f64,
) -> Result<ExamResult, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let result = db.repo.finish_exam_session(&session_id, pass_percentage)?;
    Ok(result)
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_exam_result(
    state: State<DbState>,
    session_id: String,
) -> Result<Option<ExamResult>, CommandError> {
    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;
    let result = db.repo.get_exam_result(&session_id)?;
    Ok(result)
}
