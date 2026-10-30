use crate::error::Result;
use crate::state::AppState;
use rusqlite::params;
use serde_json::json;
use tauri::State;

const KNOWN_EMULATORS: &[&str] = &[
    "retroarch", "ppsspp", "dolphin", "pcsx2", "rpcs3", "yuzu", "ryujinx",
    "mame", "fs-uae", "scummvm", "dosbox", "mednafen", "mupen64plus",
    "flycast", "duckstation", "azahar", "lime3ds", "vita3k", "xemu",
    "melonds", "desmume",
];

#[tauri::command]
pub fn detect_emudeck() -> Option<String> {
    let home = dirs::home_dir()?;
    let candidates = [
        home.join("Emulation"),
        home.join(".config/EmuDeck"),
        home.join("EmuDeck"),
    ];
    for c in candidates {
        if c.exists() {
            return Some(c.to_string_lossy().to_string());
        }
    }
    // Steam Deck default
    let deck = std::path::Path::new("/home/deck/Emulation");
    if deck.exists() {
        return Some(deck.to_string_lossy().to_string());
    }
    None
}

#[tauri::command]
pub fn detect_running_emulators() -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    // Linux /proc scan
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let comm_path = format!("/proc/{}/comm", name);
            if let Ok(comm) = std::fs::read_to_string(&comm_path) {
                let comm = comm.trim().to_lowercase();
                for emu in KNOWN_EMULATORS {
                    if comm.contains(emu) && !found.iter().any(|f| f == emu) {
                        found.push(emu.to_string());
                    }
                }
            }
            let cmdline_path = format!("/proc/{}/cmdline", name);
            if let Ok(cmd) = std::fs::read_to_string(&cmdline_path) {
                let cmd = cmd.to_lowercase().replace('\0', " ");
                for emu in KNOWN_EMULATORS {
                    if cmd.contains(emu) && !found.iter().any(|f| f == emu) {
                        found.push(emu.to_string());
                    }
                }
            }
        }
    }
    found
}

#[tauri::command]
pub fn start_session(state: State<AppState>, game_id: String) -> Result<String> {
    let conn = state.db.lock().unwrap();
    let id = crate::models::new_id();
    conn.execute(
        "INSERT INTO game_sessions (id,game_id,started_at) VALUES (?1,?2,?3)",
        params![id, game_id, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(id)
}

#[tauri::command]
pub fn stop_session(state: State<AppState>, session_id: String) -> Result<i64> {
    let conn = state.db.lock().unwrap();
    let started: String = conn.query_row(
        "SELECT started_at FROM game_sessions WHERE id=?1",
        [session_id.clone()],
        |r| r.get(0),
    )?;
    let start = chrono::DateTime::parse_from_rfc3339(&started)
        .map_err(|e| crate::error::RetroError::Other(e.to_string()))?;
    let now = chrono::Utc::now();
    let dur = (now - start.with_timezone(&chrono::Utc)).num_seconds();
    conn.execute(
        "UPDATE game_sessions SET ended_at=?1, duration_seconds=?2 WHERE id=?3",
        params![now.to_rfc3339(), dur, session_id],
    )?;
    Ok(dur)
}

#[tauri::command]
pub fn list_sessions(state: State<AppState>, game_id: Option<String>) -> Result<Vec<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    let sql = if game_id.is_some() {
        "SELECT id,game_id,started_at,ended_at,duration_seconds FROM game_sessions WHERE game_id=?1 ORDER BY started_at DESC"
    } else {
        "SELECT id,game_id,started_at,ended_at,duration_seconds FROM game_sessions ORDER BY started_at DESC LIMIT 100"
    };
    let mut stmt = conn.prepare(sql)?;
    let mut out = Vec::new();
    if let Some(gid) = &game_id {
        let rows = stmt.query_map([gid.clone()], |r| Ok(json!({
            "id": r.get::<_,String>(0)?,
            "game_id": r.get::<_,String>(1)?,
            "started_at": r.get::<_,String>(2)?,
            "ended_at": r.get::<_,Option<String>>(3)?,
            "duration_seconds": r.get::<_,Option<i64>>(4)?,
        })))?;
        for r in rows { out.push(r?); }
    } else {
        let rows = stmt.query_map([], |r| Ok(json!({
            "id": r.get::<_,String>(0)?,
            "game_id": r.get::<_,String>(1)?,
            "started_at": r.get::<_,String>(2)?,
            "ended_at": r.get::<_,Option<String>>(3)?,
            "duration_seconds": r.get::<_,Option<i64>>(4)?,
        })))?;
        for r in rows { out.push(r?); }
    }
    Ok(out)
}

#[tauri::command]
pub fn session_stats(state: State<AppState>) -> Result<serde_json::Value> {
    let conn = state.db.lock().unwrap();
    let total_sessions: i64 = conn.query_row("SELECT COUNT(*) FROM game_sessions", [], |r| r.get(0))?;
    let total_seconds: i64 = conn.query_row("SELECT COALESCE(SUM(duration_seconds),0) FROM game_sessions", [], |r| r.get(0))?;
    let top: Vec<serde_json::Value> = {
        let mut stmt = conn.prepare("SELECT game_id, COUNT(*) as c, COALESCE(SUM(duration_seconds),0) as d FROM game_sessions GROUP BY game_id ORDER BY d DESC LIMIT 10")?;
        let rows = stmt.query_map([], |r| Ok(json!({
            "game_id": r.get::<_,String>(0)?,
            "sessions": r.get::<_,i64>(1)?,
            "seconds": r.get::<_,i64>(2)?,
        })))?;
        let mut v = Vec::new();
        for r in rows { v.push(r?); }
        v
    };
    Ok(json!({
        "total_sessions": total_sessions,
        "total_seconds": total_seconds,
        "top": top,
    }))
}