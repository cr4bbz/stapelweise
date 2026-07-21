use super::CommandError;
use crate::db::models::Deck;
use crate::db::settings::AppSettings;
use crate::db::DbState;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use tauri::State;
use walkdir::WalkDir;

fn calculate_hash(s: &str) -> String {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish().to_string()
}

#[tauri::command(rename_all = "camelCase")]
pub fn sync_obsidian_vault(
    state: State<DbState>,
    vault_path: String,
    deck_name: String,
) -> Result<Deck, CommandError> {
    let canonical_path = fs::canonicalize(&vault_path).map_err(|e| {
        CommandError::from(format!("Ungültiger Vault-Pfad '{}': {}", vault_path, e))
    })?;

    if !canonical_path.is_dir() {
        return Err(CommandError::from(format!(
            "Pfad '{}' ist kein Ordner",
            vault_path
        )));
    }

    let db = state
        .lock()
        .map_err(|e| CommandError::from(format!("Lock error: {}", e)))?;

    let settings = AppSettings::load(&db.repo).unwrap_or_else(|_| AppSettings::defaults());
    let tag = settings.obsidian_flashcard_tag.clone();

    // Create or get the deck
    let deck = db.repo.get_or_create_deck(&deck_name)?;
    let deck_id = &deck.id;

    for entry in WalkDir::new(&canonical_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "md" {
                    let file_path = entry.path().to_string_lossy().into_owned();
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if content.contains(&tag) {
                            let hash = calculate_hash(&content);

                            // Parse simple markdown cards
                            let front = entry
                                .path()
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned();
                            let back = content.replace(&tag, "").trim().to_string();

                            let existing = db.repo.get_obsidian_card_hash(&file_path)?;

                            match existing {
                                Some((card_id, old_hash)) => {
                                    if old_hash != hash {
                                        if let Err(e) = db.repo.update_card(
                                            &card_id,
                                            "basic",
                                            None,
                                            None,
                                            &front,
                                            &back,
                                            vec![],
                                        ) {
                                            eprintln!("Fehler beim Aktualisieren der Obsidian-Karte {}: {}", card_id, e);
                                        }
                                        if let Err(e) = db
                                            .repo
                                            .upsert_obsidian_card(&card_id, &file_path, 0, &hash)
                                        {
                                            eprintln!("Fehler beim Speichern der Obsidian-Metadata {}: {}", card_id, e);
                                        }
                                    }
                                }
                                None => {
                                    match db.repo.create_card(
                                        deck_id,
                                        "basic",
                                        None,
                                        None,
                                        &front,
                                        &back,
                                        vec![],
                                    ) {
                                        Ok(card) => {
                                            if let Err(e) = db.repo.upsert_obsidian_card(
                                                &card.id, &file_path, 0, &hash,
                                            ) {
                                                eprintln!("Fehler beim Speichern der Obsidian-Metadata {}: {}", card.id, e);
                                            }
                                        }
                                        Err(e) => eprintln!(
                                            "Fehler beim Erstellen der Obsidian-Karte: {}",
                                            e
                                        ),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Never delete a local card merely because its source file was moved, removed, or untagged.
    // Explicit deletion remains available in Stapelweise, preserving learning history by default.

    Ok(deck)
}
