use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::models::{Card, CardState, Deck, Exam, ExamTemplate, Review};
use super::repository::Repository;
use super::settings::AppSettings;
use crate::commands::CommandError;

const MAX_BACKUP_BYTES: usize = 50 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ConflictStrategy {
    Overwrite,
    Skip,
}

impl ConflictStrategy {
    fn parse(value: &str) -> Result<Self, CommandError> {
        match value {
            "overwrite" => Ok(Self::Overwrite),
            "skip" => Ok(Self::Skip),
            _ => Err(CommandError::validation(
                "Unknown backup conflict strategy. Use 'overwrite' or 'skip'.",
            )),
        }
    }
}

fn ensure_backup_size(json_data: &str) -> Result<(), CommandError> {
    if json_data.len() > MAX_BACKUP_BYTES {
        return Err(CommandError::import_failed(
            "The backup is too large. The maximum import size is 50 MB.",
            None,
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPackage {
    pub version: u32,
    pub exported_at: String,
    pub decks: Vec<Deck>,
    pub cards: Vec<Card>,
    pub card_states: Vec<CardState>,
    pub reviews: Vec<Review>,
    pub settings: Option<AppSettings>,
    #[serde(default)]
    pub exams: Vec<Exam>,
    #[serde(default)]
    pub exam_templates: Vec<ExamTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInspection {
    pub package_version: u32,
    pub exported_at: String,
    pub deck_count: usize,
    pub card_count: usize,
    pub review_count: usize,
    pub exam_count: usize,
    pub template_count: usize,
    pub existing_deck_conflicts: Vec<String>,
    pub existing_card_conflicts: usize,
    pub warnings: Vec<String>,
}

pub fn export_backup(repo: &Repository) -> Result<BackupPackage, CommandError> {
    let decks = repo.list_all_decks()?;
    let cards = repo.list_all_cards()?;
    let reviews = repo.list_all_reviews()?;

    let mut card_states = Vec::new();
    for c in &cards {
        if let Some(state) = repo.get_card_state(&c.id)? {
            card_states.push(state);
        }
    }

    let settings = AppSettings::load(repo).ok();
    let exams = repo.list_exams(true)?;
    let exam_templates = repo.list_exam_templates()?;

    Ok(BackupPackage {
        version: 2,
        exported_at: Utc::now().to_rfc3339(),
        decks,
        cards,
        card_states,
        reviews,
        settings,
        exams,
        exam_templates,
    })
}

pub fn inspect_backup(
    repo: &Repository,
    json_data: &str,
) -> Result<ImportInspection, CommandError> {
    ensure_backup_size(json_data)?;
    let pkg: BackupPackage = serde_json::from_str(json_data)
        .map_err(|e| CommandError::import_failed("Ungültiges Backup-JSON", Some(e.to_string())))?;

    let existing_decks = repo.list_all_decks()?;
    let existing_deck_names: HashSet<String> = existing_decks.into_iter().map(|d| d.name).collect();

    let mut deck_conflicts = Vec::new();
    for d in &pkg.decks {
        if existing_deck_names.contains(&d.name) {
            deck_conflicts.push(d.name.clone());
        }
    }

    let existing_cards = repo.list_all_cards()?;
    let existing_card_ids: HashSet<String> = existing_cards.into_iter().map(|c| c.id).collect();

    let mut card_conflicts = 0;
    for c in &pkg.cards {
        if existing_card_ids.contains(&c.id) {
            card_conflicts += 1;
        }
    }

    let mut warnings = Vec::new();
    if pkg.version > 2 {
        warnings.push(format!(
            "Das Backup hat Version {}, aktuell unterstützt wird Version 2.",
            pkg.version
        ));
    }

    Ok(ImportInspection {
        package_version: pkg.version,
        exported_at: pkg.exported_at,
        deck_count: pkg.decks.len(),
        card_count: pkg.cards.len(),
        review_count: pkg.reviews.len(),
        exam_count: pkg.exams.len(),
        template_count: pkg.exam_templates.len(),
        existing_deck_conflicts: deck_conflicts,
        existing_card_conflicts: card_conflicts,
        warnings,
    })
}

pub fn restore_backup(
    repo: &Repository,
    json_data: &str,
    conflict_strategy: &str,
) -> Result<(), CommandError> {
    ensure_backup_size(json_data)?;
    let conflict_strategy = ConflictStrategy::parse(conflict_strategy)?;
    let pkg: BackupPackage = serde_json::from_str(json_data).map_err(|e| {
        CommandError::import_failed("Ungültiges Backup-JSON Format", Some(e.to_string()))
    })?;

    let existing_cards = repo.list_all_cards()?;
    let existing_card_ids: HashSet<String> = existing_cards.into_iter().map(|c| c.id).collect();
    let existing_deck_ids: HashSet<String> = repo
        .list_all_decks()?
        .into_iter()
        .map(|deck| deck.id)
        .collect();
    let existing_exam_ids: HashSet<String> = repo
        .list_exams(true)?
        .into_iter()
        .map(|exam| exam.id)
        .collect();
    let existing_template_ids: HashSet<String> = repo
        .list_exam_templates()?
        .into_iter()
        .map(|template| template.id)
        .collect();
    let mut restored_card_ids = HashSet::new();

    // Begin atomic transaction
    repo.conn().execute_batch("BEGIN TRANSACTION;")?;

    // 1. Decks
    for deck in &pkg.decks {
        if conflict_strategy == ConflictStrategy::Skip && existing_deck_ids.contains(&deck.id) {
            continue;
        }
        if let Err(e) = repo.conn().execute(
            "INSERT INTO decks (id, name, archived, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name, archived = excluded.archived, updated_at = excluded.updated_at",
            rusqlite::params![deck.id, deck.name, deck.archived, deck.created_at, deck.updated_at],
        ) {
            let _ = repo.conn().execute_batch("ROLLBACK;");
            return Err(CommandError::from(e));
        }
    }

    // 2. Cards
    for card in &pkg.cards {
        let exists = existing_card_ids.contains(&card.id);
        if exists && conflict_strategy == ConflictStrategy::Skip {
            continue;
        }

        if let Err(e) = repo.conn().execute(
            "INSERT INTO cards (id, deck_id, card_type, content, reasoning, front, back, front_language, back_language, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
               front = excluded.front,
               back = excluded.back,
               card_type = excluded.card_type,
               deck_id = excluded.deck_id,
               content = excluded.content,
               reasoning = excluded.reasoning,
               front_language = excluded.front_language,
               back_language = excluded.back_language,
               updated_at = excluded.updated_at",
            rusqlite::params![
                card.id,
                card.deck_id,
                card.card_type,
                card.content,
                card.reasoning,
                card.front,
                card.back,
                card.front_language,
                card.back_language,
                card.created_at,
                card.updated_at
            ],
        ) {
            let _ = repo.conn().execute_batch("ROLLBACK;");
            return Err(CommandError::from(e));
        }

        restored_card_ids.insert(card.id.clone());

        // Set tags with error checking & rollback
        if !card.tags.is_empty() {
            if let Err(e) = repo.set_card_tags(&card.id, &card.tags) {
                let _ = repo.conn().execute_batch("ROLLBACK;");
                return Err(CommandError::from(e));
            }
        }
    }

    // 3. CardStates
    for cs in &pkg.card_states {
        if conflict_strategy == ConflictStrategy::Skip && !restored_card_ids.contains(&cs.card_id) {
            continue;
        }
        if let Err(e) = repo.conn().execute(
            "INSERT INTO card_state (card_id, interval, ease_factor, repetitions, next_review, total_reviews, correct_streak, last_review)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(card_id) DO UPDATE SET
               interval = excluded.interval,
               ease_factor = excluded.ease_factor,
               repetitions = excluded.repetitions,
               next_review = excluded.next_review,
               total_reviews = excluded.total_reviews,
               correct_streak = excluded.correct_streak,
               last_review = excluded.last_review",
            rusqlite::params![
                cs.card_id,
                cs.interval,
                cs.ease_factor,
                cs.repetitions,
                cs.next_review,
                cs.total_reviews,
                cs.correct_streak,
                cs.last_review
            ],
        ) {
            let _ = repo.conn().execute_batch("ROLLBACK;");
            return Err(CommandError::from(e));
        }
    }

    // 4. Reviews
    for rev in &pkg.reviews {
        if conflict_strategy == ConflictStrategy::Skip && !restored_card_ids.contains(&rev.card_id)
        {
            continue;
        }
        if let Err(e) = repo.conn().execute(
            "INSERT INTO reviews (id, card_id, quality, reviewed_at, interval, ease_factor, repetitions, prev_state)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO NOTHING",
            rusqlite::params![
                rev.id,
                rev.card_id,
                rev.quality,
                rev.reviewed_at,
                rev.interval,
                rev.ease_factor,
                rev.repetitions,
                rev.prev_state
            ],
        ) {
            let _ = repo.conn().execute_batch("ROLLBACK;");
            return Err(CommandError::from(e));
        }
    }

    // 5. Exams and their deck assignments
    for exam in &pkg.exams {
        if conflict_strategy == ConflictStrategy::Skip && existing_exam_ids.contains(&exam.id) {
            continue;
        }
        if let Err(e) = repo.conn().execute(
            "INSERT INTO exams (id, name, exam_type, exam_date, archived, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name,
               exam_type = excluded.exam_type,
               exam_date = excluded.exam_date,
               archived = excluded.archived",
            rusqlite::params![
                exam.id,
                exam.name,
                exam.exam_type,
                exam.exam_date,
                exam.archived,
                exam.created_at
            ],
        ) {
            let _ = repo.conn().execute_batch("ROLLBACK;");
            return Err(CommandError::from(e));
        }

        if let Err(e) = repo.conn().execute(
            "DELETE FROM exam_decks WHERE exam_id = ?1",
            rusqlite::params![exam.id],
        ) {
            let _ = repo.conn().execute_batch("ROLLBACK;");
            return Err(CommandError::from(e));
        }

        for deck_id in &exam.deck_ids {
            if let Err(e) = repo.conn().execute(
                "INSERT INTO exam_decks (exam_id, deck_id) VALUES (?1, ?2)",
                rusqlite::params![exam.id, deck_id],
            ) {
                let _ = repo.conn().execute_batch("ROLLBACK;");
                return Err(CommandError::from(e));
            }
        }
    }

    // 6. ExamTemplates
    for tmpl in &pkg.exam_templates {
        if conflict_strategy == ConflictStrategy::Skip && existing_template_ids.contains(&tmpl.id) {
            continue;
        }
        let deck_ids_json =
            serde_json::to_string(&tmpl.deck_ids).unwrap_or_else(|_| "[]".to_string());
        let tags_json = serde_json::to_string(&tmpl.tags).unwrap_or_else(|_| "[]".to_string());
        let allowed_types_json =
            serde_json::to_string(&tmpl.allowed_card_types).unwrap_or_else(|_| "[]".to_string());

        if let Err(e) = repo.conn().execute(
            "INSERT INTO exam_templates (id, name, deck_ids_json, tags_json, allowed_types_json, question_count, time_limit_minutes, pass_percentage, seed, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name,
               deck_ids_json = excluded.deck_ids_json,
               tags_json = excluded.tags_json,
               allowed_types_json = excluded.allowed_types_json,
               question_count = excluded.question_count,
               time_limit_minutes = excluded.time_limit_minutes,
               pass_percentage = excluded.pass_percentage,
               seed = excluded.seed",
            rusqlite::params![
                tmpl.id,
                tmpl.name,
                deck_ids_json,
                tags_json,
                allowed_types_json,
                tmpl.question_count,
                tmpl.time_limit_minutes,
                tmpl.pass_percentage,
                tmpl.seed,
                tmpl.created_at
            ],
        ) {
            let _ = repo.conn().execute_batch("ROLLBACK;");
            return Err(CommandError::from(e));
        }
    }

    // 7. Settings
    if conflict_strategy == ConflictStrategy::Overwrite {
        if let Some(ref s) = pkg.settings {
            if let Err(e) = s.save(repo) {
                let _ = repo.conn().execute_batch("ROLLBACK;");
                return Err(CommandError::from(e));
            }
        }
    }

    repo.conn().execute_batch("COMMIT;")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ensure_backup_size, ConflictStrategy, MAX_BACKUP_BYTES};

    #[test]
    fn accepts_only_documented_conflict_strategies() {
        assert_eq!(
            ConflictStrategy::parse("overwrite").unwrap(),
            ConflictStrategy::Overwrite
        );
        assert_eq!(
            ConflictStrategy::parse("skip").unwrap(),
            ConflictStrategy::Skip
        );
        assert!(ConflictStrategy::parse("merge").is_err());
    }

    #[test]
    fn rejects_oversized_backup_payloads() {
        assert!(ensure_backup_size(&"x".repeat(MAX_BACKUP_BYTES)).is_ok());
        assert!(ensure_backup_size(&"x".repeat(MAX_BACKUP_BYTES + 1)).is_err());
    }
}
