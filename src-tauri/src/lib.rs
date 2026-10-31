pub mod commands;
pub mod db;
pub mod error;
pub mod identify;
pub mod metadata;
pub mod models;
pub mod remote;
pub mod scanner;
pub mod state;

use state::AppState;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let app_state = AppState::new().expect("failed to init database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::list_sources,
            commands::add_source,
            commands::remove_source,
            commands::scan_source,
            commands::scan_source_cmd,
            commands::scan_local_folder,
            commands::scan_remote_folder,
            commands::cancel_scan,
            commands::test_remote_connection,
            commands::add_remote_source,
            commands::list_remote_dir,
            commands::scan_remote_source,
            commands::copy_remote_files,
            commands::list_games,
            commands::detect_emudeck,
            commands::detect_running_emulators,
            commands::start_session,
            commands::stop_session,
            commands::list_sessions,
            commands::session_stats,
            commands::compute_hashes,
            commands::compute_hashes_for_game,
            commands::export_backup,
            commands::import_backup,
            commands::list_transfer_jobs,
            commands::list_metadata_providers,
            commands::enrich_game,
            commands::get_game_metadata,
            commands::get_game,
            commands::search_games_fts,
            commands::find_duplicates,
            commands::list_collections,
            commands::create_collection,
            commands::add_game_to_collection,
            commands::remove_game_from_collection,
            commands::collection_games,
            commands::correct_game_title,
            commands::link_file_to_game,
            commands::stats,
            commands::list_platforms,
            commands::app_info,
            commands::reset_database,
            commands::clear_library,
            commands::get_system_locale,
        ])
        .setup(|app| {
            let _ = app;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
