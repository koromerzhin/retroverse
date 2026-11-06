use crate::dat;
use crate::error::{Result, RetroError};
use crate::state::AppState;
use rusqlite::params;
use std::fs;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn import_dat(state: State<AppState>, path: String) -> Result<usize> {
    let content = fs::read_to_string(&path).map_err(RetroError::Io)?;
    let parsed = dat::parse_dat(&content)?;
    let conn = state.db.lock().unwrap();
    let now = chrono::Utc::now().to_rfc3339();
    let dat_id = Uuid::new_v4().to_string();
    let name = std::path::Path::new(&path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("dat")
        .to_string();
    conn.execute(
        "INSERT INTO dat_files (id,name,system,version,imported_at) VALUES (?1,?2,?3,?4,?5)",
        params![dat_id, name, parsed.system, parsed.version, now],
    )?;
    let mut count = 0usize;
    for e in &parsed.entries {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO dat_entries (id,dat_id,name,size,crc32,md5,sha1,sha256) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![id, dat_id, e.name, e.size, e.crc32, e.md5, e.sha1, e.sha256],
        )?;
        count += 1;
    }
    Ok(count)
}

#[tauri::command]
pub fn verify_library(state: State<AppState>) -> Result<usize> {
    let conn = state.db.lock().unwrap();
    let now = chrono::Utc::now().to_rfc3339();
    // For each game, find a matching dat_entry by hash (sha1 or md5 or crc32 or sha256).
    let game_ids: Vec<String> = {
        let mut stmt = conn.prepare("SELECT id FROM games")?;
        let rows = stmt.query_map([], |r| r.get(0))?;
        rows.filter_map(|r| r.ok()).collect()
    };
    let mut verified = 0usize;
    for gid in game_ids {
        let hashes: (Option<String>, Option<String>, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT hash, md5, sha1, crc32 FROM game_files gf JOIN game_file_links l ON l.file_id=gf.id WHERE l.game_id=?1 LIMIT 1",
                [gid.clone()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap_or((None, None, None, None));
        let (sha256, md5, sha1, crc32) = hashes;
        // find matching dat entry
        let mut matched: Option<String> = None;
        if let Some(s) = &sha256 {
            matched = conn
                .query_row(
                    "SELECT id FROM dat_entries WHERE sha256=?1 LIMIT 1",
                    [s.clone()],
                    |r| r.get(0),
                )
                .ok();
        }
        if matched.is_none()
            && let Some(s) = &sha1
        {
            matched = conn
                .query_row(
                    "SELECT id FROM dat_entries WHERE sha1=?1 LIMIT 1",
                    [s.clone()],
                    |r| r.get(0),
                )
                .ok();
        }
        if matched.is_none()
            && let Some(s) = &md5
        {
            matched = conn
                .query_row(
                    "SELECT id FROM dat_entries WHERE md5=?1 LIMIT 1",
                    [s.clone()],
                    |r| r.get(0),
                )
                .ok();
        }
        if matched.is_none()
            && let Some(s) = &crc32
        {
            matched = conn
                .query_row(
                    "SELECT id FROM dat_entries WHERE crc32=?1 LIMIT 1",
                    [s.clone()],
                    |r| r.get(0),
                )
                .ok();
        }
        let status = if matched.is_some() {
            "verified"
        } else {
            "unknown"
        };
        if matched.is_some() {
            verified += 1;
        }
        conn.execute(
            "INSERT OR REPLACE INTO game_preservation (game_id, status, dat_entry_id, verified_at, notes) VALUES (?1,?2,?3,?4,NULL)",
            params![gid, status, matched, now],
        )?;
    }
    Ok(verified)
}

#[tauri::command]
pub fn get_preservation_status(state: State<AppState>, game_id: String) -> Result<String> {
    let conn = state.db.lock().unwrap();
    let status: String = conn
        .query_row(
            "SELECT status FROM game_preservation WHERE game_id=?1",
            [game_id],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "unknown".to_string());
    Ok(status)
}
