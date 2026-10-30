pub mod commands;
pub mod db;
pub mod error;
pub mod identify;
pub mod models;
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
            commands::list_games,
            commands::get_game,
            commands::correct_game_title,
            commands::link_file_to_game,
            commands::stats,
            commands::list_platforms,
            commands::app_info,
        ])
        .setup(|app| {
            let _ = app;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
