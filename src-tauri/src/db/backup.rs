use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::models::{Card, CardState, Deck, Exam, ExamTemplate, Review};
use super::repository::Repository;
use super::settings::AppSettings;
use crate::commands::CommandError;

const CURRENT_BACKUP_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStrategy {
    Overwrite,
    Skip,
    Merge,
}

impl TryFrom<&str> for ConflictStrategy {
    type Error = CommandError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "overwrite" => Ok(Self::Overwrite),
            "skip" => Ok(Self::Skip),
            "merge" => Ok(Self::Merge),
            _ => Err(CommandError::validation(
                "conflict_strategy must be overwrite, skip or merge",
            )),
        }
    }
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

fn parse_package(json_data: &str) -> Result<BackupPackage, CommandError> {
    serde_json::from_str(json_data)
        .map_err(|error| CommandError::import_failed("Ungültiges Backup-JSON", Some(error.to_string())))
}

fn ensure_supported_version(pkg: &BackupPackage) -> Result<(), CommandError> {
    if pkg.version > CURRENT_BACKUP_VERSION {
        return Err(CommandError::import_failed(
            "Backup-Version wird von dieser Stapelweise-Version nicht unterstützt",
            Some(format!(
                "Backup-Version: {}, unterstützt bis: {}",
                pkg.version, CURRENT_BACKUP_VERSION
            )),
        ));
    }
    Ok(())
}

pub fn export_backup(repo: &Repository) -> Result<BackupPackage, CommandError> {
    let decks = repo.list_all_decks()?;
    let cards = repo.list_all_cards()?;
    let reviews = repo.list_all_reviews()?;

    let mut card_states = Vec::new();
    for card in &cards {
        if let Some(state) = repo.get_card_state(&card.id)? {
            card_states.push(state);
        }
    }

    let settings = AppSettings::load(repo).ok();
    let exams = repo.list_exams(true)?;
    let exam_templates = repo.list_exam_templates()?;

    Ok(BackupPackage {
        version: CURRENT_BACKUP_VERSION,
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
    let pkg = parse_package(json_data)?;
    ensure_supported_version(&pkg)?;

    let existing_deck_ids: HashSet<String> = repo
        .list_all_decks()?
        .into_iter()
        .map(|deck| deck.id)
        .collect();
    let existing_card_ids: HashSet<String> = repo
        .list_all_cards()?
        .into_iter()
        .map(|card| card.id)
        .collect();

    let existing_deck_conflicts = pkg
        .decks
        .iter()
        .filter(|deck| existing_deck_ids.contains(&deck.id))
        .map(|deck| deck.name.clone())
        .collect();
    let existing_card_conflicts = pkg
        .cards
        .iter()
        .filter(|card| existing_card_ids.contains(&card.id))
        .count();

    let mut warnings = Vec::new();
    if pkg.version < CURRENT_BACKUP_VERSION {
        warnings.push(format!(
            "Älteres Backup-Format (Version {}) wird kompatibel importiert.",
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
        existing_deck_conflicts,
        existing_card_conflicts,
        warnings,
    })
}

pub fn restore_backup(
    repo: &Repository,
    json_data: &str,
    conflict_strategy: &str,
) -> Result<(), CommandError> {
    let pkg = parse_package(json_data)?;
    ensure_supported_version(&pkg)?;
    let strategy = ConflictStrategy::try_from(conflict_strategy)?;

    let existing_deck_ids: HashSet<String> = repo
        .list_all_decks()?
        .into_iter()
        .map(|deck| deck.id)
        .collect();
    let existing_card_ids: HashSet<String> = repo
        .list_all_cards()?
        .into_iter()
        .map(|card| card.id)
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

    repo.conn().execute_batch("BEGIN IMMEDIATE;")?;
    let result = restore_all(
        repo,
        &pkg,
        strategy,
        &existing_deck_ids,
        &existing_card_ids,
        &existing_exam_ids,
        &existing_template_ids,
    );

    match result {
        Ok(()) => {
            repo.conn().execute_batch("COMMIT;")?;
            Ok(())
        }
        Err(error) => {
            let _ = repo.conn().execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

fn restore_all(
    repo: &Repository,
    pkg: &BackupPackage,
    strategy: ConflictStrategy,
    existing_deck_ids: &HashSet<String>,
    existing_card_ids: &HashSet<String>,
    existing_exam_ids: &HashSet<String>,
    existing_template_ids: &HashSet<String>,
) -> Result<(), CommandError> {
    for deck in &pkg.decks {
        if strategy == ConflictStrategy::Skip && existing_deck_ids.contains(&deck.id) {
            continue;
        }
        repo.conn().execute(
            "INSERT INTO decks (id, name, archived, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name, archived = excluded.archived, updated_at = excluded.updated_at",
            rusqlite::params![deck.id, deck.name, deck.archived, deck.created_at, deck.updated_at],
        )?;
    }

    for card in &pkg.cards {
        let existed = existing_card_ids.contains(&card.id);
        if strategy == ConflictStrategy::Skip && existed {
            continue;
        }

        repo.conn().execute(
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
        )?;
        repo.set_card_tags(&card.id, &card.tags)?;
    }

    for state in &pkg.card_states {
        let existed = existing_card_ids.contains(&state.card_id);
        if existed && matches!(strategy, ConflictStrategy::Skip | ConflictStrategy::Merge) {
            continue;
        }
        repo.conn().execute(
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
                state.card_id,
                state.interval,
                state.ease_factor,
                state.repetitions,
                state.next_review,
                state.total_reviews,
                state.correct_streak,
                state.last_review
            ],
        )?;
    }

    for review in &pkg.reviews {
        if existing_card_ids.contains(&review.card_id)
            && matches!(strategy, ConflictStrategy::Skip | ConflictStrategy::Merge)
        {
            continue;
        }
        repo.conn().execute(
            "INSERT INTO reviews (id, card_id, quality, reviewed_at, interval, ease_factor, repetitions, prev_state)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO NOTHING",
            rusqlite::params![
                review.id,
                review.card_id,
                review.quality,
                review.reviewed_at,
                review.interval,
                review.ease_factor,
                review.repetitions,
                review.prev_state
            ],
        )?;
    }

    for exam in &pkg.exams {
        let existed = existing_exam_ids.contains(&exam.id);
        if strategy == ConflictStrategy::Skip && existed {
            continue;
        }
        repo.conn().execute(
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
        )?;
        repo.conn().execute(
            "DELETE FROM exam_decks WHERE exam_id = ?1",
            rusqlite::params![exam.id],
        )?;
        for deck_id in &exam.deck_ids {
            repo.conn().execute(
                "INSERT INTO exam_decks (exam_id, deck_id) VALUES (?1, ?2)",
                rusqlite::params![exam.id, deck_id],
            )?;
        }
    }

    for template in &pkg.exam_templates {
        if strategy == ConflictStrategy::Skip && existing_template_ids.contains(&template.id) {
            continue;
        }
        let deck_ids_json = serde_json::to_string(&template.deck_ids)?;
        let tags_json = serde_json::to_string(&template.tags)?;
        let allowed_types_json = serde_json::to_string(&template.allowed_card_types)?;
        repo.conn().execute(
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
                template.id,
                template.name,
                deck_ids_json,
                tags_json,
                allowed_types_json,
                template.question_count,
                template.time_limit_minutes,
                template.pass_percentage,
                template.seed,
                template.created_at
            ],
        )?;
    }

    // overwrite is the only strategy that replaces local settings. Merge explicitly
    // keeps local learning progress and local settings while backup card content wins.
    if strategy == ConflictStrategy::Overwrite {
        if let Some(settings) = &pkg.settings {
            settings.save(repo)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn empty_package(version: u32) -> BackupPackage {
        BackupPackage {
            version,
            exported_at: "2026-08-17T00:00:00Z".into(),
            decks: vec![],
            cards: vec![],
            card_states: vec![],
            reviews: vec![],
            settings: None,
            exams: vec![],
            exam_templates: vec![],
        }
    }

    fn test_repo() -> Repository {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();
        Repository::new(conn)
    }

    #[test]
    fn parses_all_conflict_strategies() {
        assert_eq!(ConflictStrategy::try_from("overwrite").unwrap(), ConflictStrategy::Overwrite);
        assert_eq!(ConflictStrategy::try_from("skip").unwrap(), ConflictStrategy::Skip);
        assert_eq!(ConflictStrategy::try_from("merge").unwrap(), ConflictStrategy::Merge);
    }

    #[test]
    fn rejects_unknown_conflict_strategy() {
        assert!(ConflictStrategy::try_from("surprise").is_err());
    }

    #[test]
    fn rejects_newer_backup_version_during_inspection() {
        let repo = test_repo();
        let json = serde_json::to_string(&empty_package(CURRENT_BACKUP_VERSION + 1)).unwrap();
        assert!(inspect_backup(&repo, &json).is_err());
    }

    #[test]
    fn rejects_newer_backup_version_during_restore() {
        let repo = test_repo();
        let json = serde_json::to_string(&empty_package(CURRENT_BACKUP_VERSION + 1)).unwrap();
        assert!(restore_backup(&repo, &json, "merge").is_err());
    }

    #[test]
    fn accepts_current_backup_version() {
        let repo = test_repo();
        let json = serde_json::to_string(&empty_package(CURRENT_BACKUP_VERSION)).unwrap();
        let inspection = inspect_backup(&repo, &json).unwrap();
        assert_eq!(inspection.package_version, CURRENT_BACKUP_VERSION);
    }
}
