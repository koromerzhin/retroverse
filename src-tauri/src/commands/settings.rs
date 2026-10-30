use crate::error::Result;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct AppInfo {
    pub version: String,
    pub db_path: String,
}

#[tauri::command]
pub fn app_info(state: State<AppState>) -> Result<AppInfo> {
    let _ = state;
    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        db_path: crate::db::db_path().to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn reset_database(state: State<AppState>) -> Result<()> {
    let conn = state.db.lock().unwrap();
    let tables = [
        "transfer_items",
        "transfer_jobs",
        "sync_jobs",
        "metadata_jobs",
        "media_assets",
        "metadata_records",
        "game_sessions",
        "collection_games",
        "collections",
        "game_relations",
        "user_corrections",
        "game_file_links",
        "game_files",
        "games",
        "game_sources",
        "remote_connections",
    ];
    conn.execute("PRAGMA foreign_keys = OFF", [])?;
    for t in tables {
        conn.execute(&format!("DELETE FROM {}", t), [])?;
    }
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    Ok(())
}
