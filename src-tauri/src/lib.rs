pub mod commands;
pub mod config;
pub mod db;
pub mod error;
pub mod hashing;
pub mod identify;
pub mod metadata;
pub mod models;
pub mod remote;
pub mod scanner;
pub mod state;
pub mod tray;

use state::AppState;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
const SPLASH_HTML: &str = include_str!("../../public/splash.html");

#[cfg(unix)]
fn single_instance_socket_path() -> std::path::PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("retroverse").join("single-instance.sock")
}

pub fn run() {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::net::UnixStream;
        let sock_path = single_instance_socket_path();
        // If an instance is already running, notify it and exit.
        if let Ok(mut stream) = UnixStream::connect(&sock_path) {
            let _ = stream.write_all(b"activate\n");
            return;
        }
        // Remove stale socket file if present.
        let _ = std::fs::remove_file(&sock_path);
    }

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
            commands::sync_entities_from_metadata,
            commands::delete_genre,
            commands::update_genre,
            commands::add_genre,
            commands::list_genres_full,
            commands::list_classifications_full,
            commands::list_game_classifications,
            commands::sync_classifications,
            commands::delete_publisher,
            commands::update_publisher,
            commands::add_publisher,
            commands::list_publishers_full,
            commands::sync_publishers,
            commands::sync_developers,
            commands::delete_developer,
            commands::update_developer,
            commands::add_developer,
            commands::list_developers_full,
            commands::app_ready,
            commands::quit_app,
            commands::show_notification,
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
            commands::compute_hashes_for_file,
            commands::compute_hashes_for_platform,
            commands::unhashed_counts_by_platform,
            commands::compute_hashes_for_game,
            commands::compute_hashes_for_all_games,
            commands::cancel_hash,
            commands::list_db_backups,
            commands::restore_db_backup,
            commands::export_backup,
            commands::import_backup,
            commands::list_transfer_jobs,
            commands::list_metadata_providers,
            commands::enrich_game,
            commands::get_game_metadata,
            commands::list_genres,
            commands::sync_genres,
            commands::list_developers,
            commands::list_publishers,
            commands::set_game_metadata,
            commands::save_game_info,
            commands::get_game,
            commands::search_games_fts,
            commands::platform_game_counts,
            commands::platform_file_counts,
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
            use base64::Engine;
            use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

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

            // Sync genres from ScreenScraper in background.
            let state = app.state::<crate::state::AppState>().inner().clone();
            tauri::async_runtime::spawn(async move {
                let _ = crate::commands::genres::sync_genres_inner(&state).await;
                let _ = crate::commands::entities::sync_entities_from_metadata_inner(&state);
            });

            // Single-instance listener (Unix): restore window when another instance pings.
            #[cfg(unix)]
            {
                use std::os::unix::net::UnixListener;
                let sock_path = single_instance_socket_path();
                if let Ok(listener) = UnixListener::bind(&sock_path) {
                    let app_handle = app.handle().clone();
                    std::thread::spawn(move || {
                        for mut s in listener.incoming().flatten() {
                            use std::io::Read;
                            let mut buf = [0u8; 64];
                            let _ = s.read(&mut buf);
                            if let Some(w) = app_handle.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.unminimize();
                                let _ = w.set_focus();
                            }
                        }
                    });
                }
            }

            // Force the main webview to load, then hide it so only the splashscreen is visible.
            let _ = main_window.show();
            let _ = main_window.set_position(tauri::PhysicalPosition::new(-10000, -10000));
            let _ = main_window.hide();

            // Independent splashscreen window with embedded HTML (data URL, no file/URL resolution).
            let splash_url = format!(
                "data:text/html;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(SPLASH_HTML)
            );
            let _splash = WebviewWindowBuilder::new(
                app,
                "splashscreen",
                WebviewUrl::External(splash_url.parse().unwrap()),
            )
            .title("RETROVERSE")
            .inner_size(440.0, 280.0)
            .resizable(false)
            .decorations(false)
            .center()
            .always_on_top(true)
            .visible(true)
            .build()?;

            // Backend-driven transition: keep splash visible 15s, then show main and close splash.
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(15)).await;
                if let Some(main) = app_handle.get_webview_window("main") {
                    let _ = main.show();
                }
                // Splash stays until frontend signals readiness via app_ready.
            });

            tracing::info!(
                "setup: main present={}, splash present={}",
                app.get_webview_window("main").is_some(),
                app.get_webview_window("splashscreen").is_some()
            );

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    #[cfg(unix)]
    {
        let _ = std::fs::remove_file(single_instance_socket_path());
    }
}
