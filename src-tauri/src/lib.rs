mod commands;
mod db;
pub mod mcp;
mod seed;
mod srs;
pub mod test_engine;

use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}));
    }

    builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                app.deep_link().register_all()?;
            }
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            let database = Database::new(app_data_dir).expect("failed to initialize database");

            app.manage(db::DbState::new(database));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::decks::create_deck,
            commands::decks::list_decks,
            commands::decks::get_deck,
            commands::decks::update_deck,
            commands::decks::delete_deck,
            commands::decks::archive_deck,
            commands::decks::restore_deck,
            commands::decks::import_deck,
            commands::exams::create_exam,
            commands::exams::list_exams,
            commands::exams::update_exam,
            commands::exams::delete_exam,
            commands::exams::archive_exam,
            commands::exams::restore_exam,
            commands::exams::get_exam_stats,
            commands::test_engine::create_exam_template,
            commands::test_engine::list_exam_templates,
            commands::test_engine::get_exam_template,
            commands::test_engine::delete_exam_template,
            commands::test_engine::start_exam_session,
            commands::test_engine::get_exam_session_with_questions,
            commands::test_engine::submit_exam_question_answer,
            commands::test_engine::finish_exam_session,
            commands::test_engine::get_exam_result,
            commands::backup::export_backup_data,
            commands::backup::inspect_backup_data,
            commands::backup::restore_backup_data,
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
            commands::integrations::import_zotero_local,
            commands::integrations::import_notion_data_source,
            commands::integrations::import_moodle_glossary,
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

            let database = Database::new(app_data_dir).expect("failed to initialize database");

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
