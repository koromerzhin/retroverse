use crate::error::{Result, RetroError};
use crate::models::{GameSource, ScanResult};
use crate::scanner;
use crate::state::AppState;
use std::sync::atomic::Ordering;
use chrono::Utc;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn list_sources(state: State<AppState>) -> Result<Vec<GameSource>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id,name,kind,path,created_at,last_scan_at,connection_id,remote_path,available,last_error FROM game_sources ORDER BY created_at DESC")?;
    let rows = stmt.query_map([], |r| {
        let created: String = r.get(4)?;
        let last: Option<String> = r.get(5)?;
        let avail: i64 = r.get(8).unwrap_or(1);
        Ok(GameSource {
            id: r.get(0)?,
            name: r.get(1)?,
            kind: r.get(2)?,
            path: r.get(3)?,
            created_at: created.parse().unwrap_or_else(|_| Utc::now()),
            last_scan_at: last.and_then(|s| s.parse().ok()),
            connection_id: r.get(6)?,
            remote_path: r.get(7)?,
            available: avail != 0,
            last_error: r.get(9)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[tauri::command]
pub fn add_source(state: State<AppState>, name: String, path: String) -> Result<GameSource> {
    // validate path
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(RetroError::Invalid(format!(
            "path does not exist: {}",
            path
        )));
    }
    if !p.is_dir() {
        return Err(RetroError::Invalid("path must be a directory".into()));
    }
    // A source must point to a folder containing a 'roms' subfolder (or be 'roms' itself).
    let final_path = if p.ends_with("roms") {
        path.clone()
    } else {
        let roms = p.join("roms");
        if roms.is_dir() {
            roms.to_string_lossy().to_string()
        } else {
            return Err(RetroError::Invalid(
                "Le dossier doit contenir un sous-dossier 'roms'".into(),
            ));
        }
    };

    let id = crate::models::new_id();
    let now = Utc::now();
    {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO game_sources (id,name,kind,path,created_at) VALUES (?1,?2,?3,?4,?5)",
            params![id, name, "local", final_path, now.to_rfc3339()],
        )?;
    }
    Ok(GameSource {
        id,
        name,
        kind: "local".into(),
        path: final_path,
        created_at: now,
        last_scan_at: None,
        connection_id: None,
        remote_path: None,
        available: true,
        last_error: None,
    })
}

#[tauri::command]
pub fn remove_source(state: State<AppState>, id: String) -> Result<()> {
    let conn = state.db.lock().unwrap();
    conn.execute("DELETE FROM game_sources WHERE id = ?1", [id])?;
    Ok(())
}

#[tauri::command]
pub fn scan_source(state: State<AppState>, app: tauri::AppHandle, source_id: String, folder: Option<String>) -> Result<ScanResult> {
    state.cancel_scan.store(false, Ordering::SeqCst);
    let mut conn = state.db.lock().unwrap();
    scanner::scan_source(&mut conn, &source_id, Some(app), state.cancel_scan.clone(), folder)
}

#[tauri::command]
pub fn scan_local_folder(state: State<AppState>, app: tauri::AppHandle, source_id: String, folder_path: String) -> Result<ScanResult> {
    state.cancel_scan.store(false, Ordering::SeqCst);
    let mut conn = state.db.lock().unwrap();
    scanner::scan_source(&mut conn, &source_id, Some(app), state.cancel_scan.clone(), Some(folder_path))
}

#[tauri::command]
pub fn cancel_scan(state: State<AppState>) -> Result<()> {
    state.cancel_scan.store(true, Ordering::SeqCst);
    Ok(())
}
