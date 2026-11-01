pub mod commands;
pub mod config;
pub mod db;
pub mod error;
pub mod identify;
pub mod metadata;
pub mod models;
pub mod remote;
pub mod scanner;
pub mod state;
pub mod tray;

use state::AppState;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // File-based logging (readable by external apps). Logs go to <data_dir>/retroverse/logs/
    let log_dir = crate::db::db_path()
        .parent()
        .map(|p| p.join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("logs"));
    let _ = std::fs::create_dir_all(&log_dir);
    let file_appender = tracing_appender::rolling::daily(log_dir.clone(), "retroverse.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    // Leak the guard so the writer stays alive for the process lifetime.
    Box::leak(Box::new(_guard));

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    tracing::info!("RETROVERSE starting; logs at {:?}", log_dir);

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
            commands::platform_game_counts,
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
            commands::get_provider_config,
            commands::set_provider_config,
            commands::import_gamelists,
        ])
        .setup(|app| {
            use tauri::Manager;

            // Minimize to tray (background): hide window on close instead of exiting.
            let main_window = app.get_webview_window("main").unwrap();
            let mw = main_window.clone();
            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = mw.hide();
                }
            });

            // Spawn system tray (KDE/freedesktop StatusNotifierItem, pure Rust).
            let handle = app.handle().clone();
            crate::tray::spawn_tray(handle);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
