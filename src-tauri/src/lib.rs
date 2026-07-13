mod commands;
mod db;
mod seed;
mod srs;
pub mod mcp;

use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            let database =
                Database::new(app_data_dir).expect("failed to initialize database");

            app.manage(db::DbState::new(database));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::decks::create_deck,
            commands::decks::list_decks,
            commands::decks::get_deck,
            commands::decks::update_deck,
            commands::decks::delete_deck,
            commands::decks::import_deck,
            commands::exams::create_exam,
            commands::exams::list_exams,
            commands::exams::update_exam,
            commands::exams::delete_exam,
            commands::exams::get_exam_stats,
            commands::cards::create_card,
            commands::cards::list_cards,
            commands::cards::update_card,
            commands::cards::delete_card,
            commands::cards::get_due_cards,
            commands::cards::count_due_cards,
            commands::cards::get_due_cards_by_tags,
            commands::cards::count_total_cards,
            commands::cards::submit_review,
            commands::cards::search_cards,
            commands::cards::undo_last_review,
            commands::cards::get_card_state,
            commands::cards::get_all_tags,
            commands::seed::seed_sample_data,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::stats::get_deck_stats,
            commands::stats::get_dashboard_stats,
            commands::obsidian::sync_obsidian_vault,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn run_mcp() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            let database =
                Database::new(app_data_dir).expect("failed to initialize database");

            std::thread::spawn(move || {
                mcp::run_mcp_loop(database);
                std::process::exit(0);
            });

            // Close the main window to run invisibly
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.close();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application in MCP mode");
}
