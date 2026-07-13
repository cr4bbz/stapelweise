use crate::db::DbState;
use crate::db::models::Deck;
use crate::db::settings::AppSettings;
use super::CommandError;
use tauri::State;
use walkdir::WalkDir;
use std::fs;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;

fn calculate_hash(s: &str) -> String {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish().to_string()
}

#[tauri::command(rename_all = "camelCase")]
pub fn sync_obsidian_vault(state: State<DbState>, vault_path: String, deck_name: String) -> Result<Deck, CommandError> {
    let db = state.lock().map_err(|e| CommandError(format!("Lock error: {}", e)))?;
    
    let settings = AppSettings::load(&db.repo).unwrap_or_else(|_| AppSettings::defaults());
    let tag = settings.obsidian_flashcard_tag.clone();

    // Create or get the deck
    let deck = db.repo.get_or_create_deck(&deck_name)?;
    let deck_id = &deck.id;

    let mut seen_paths = HashSet::new();

    for entry in WalkDir::new(&vault_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "md" {
                    let file_path = entry.path().to_string_lossy().into_owned();
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if content.contains(&tag) {
                            seen_paths.insert(file_path.clone());
                            let hash = calculate_hash(&content);
                            
                            // Parse simple markdown cards
                            let front = entry.path().file_stem().unwrap_or_default().to_string_lossy().into_owned();
                            let back = content.replace(&tag, "").trim().to_string();

                            let existing = db.repo.get_obsidian_card_hash(&file_path)?;

                            match existing {
                                Some((card_id, old_hash)) => {
                                    if old_hash != hash {
                                        let _ = db.repo.update_card(&card_id, "basic", None, None, &front, &back, vec![]);
                                        let _ = db.repo.upsert_obsidian_card(&card_id, &file_path, 0, &hash);
                                    }
                                }
                                None => {
                                    if let Ok(card) = db.repo.create_card(deck_id, "basic", None, None, &front, &back, vec![]) {
                                        let _ = db.repo.upsert_obsidian_card(&card.id, &file_path, 0, &hash);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Prune deleted cards
    if let Ok(all_paths) = db.repo.get_all_obsidian_file_paths() {
        for path in all_paths {
            if !seen_paths.contains(&path) {
                // Was deleted or tag removed
                if let Ok(Some((card_id, _))) = db.repo.get_obsidian_card_hash(&path) {
                    let _ = db.repo.delete_card(&card_id);
                }
            }
        }
    }

    Ok(deck)
}
