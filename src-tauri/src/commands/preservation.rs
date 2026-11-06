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

#[tauri::command]
pub fn list_preservation_statuses(state: State<AppState>) -> Result<Vec<(String, String)>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT game_id, status FROM game_preservation")?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[tauri::command]
pub fn preservation_summary(state: State<AppState>) -> Result<Vec<(String, i64)>> {
    let conn = state.db.lock().unwrap();
    let mut stmt =
        conn.prepare("SELECT status, COUNT(*) FROM game_preservation GROUP BY status")?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[tauri::command]
pub fn export_preservation_pack(state: State<AppState>, dest_path: String) -> Result<usize> {
    use std::io::Write;
    let conn = state.db.lock().unwrap();
    // Gather games with their files and metadata.
    let mut stmt =
        conn.prepare("SELECT id, title, platform_id, metadata_json FROM games ORDER BY title")?;
    let games: Vec<(String, String, Option<String>, Option<String>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .filter_map(|r| r.ok())
        .collect();

    let file = std::fs::File::create(&dest_path).map_err(RetroError::Io)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut manifest_games = Vec::new();
    let mut count = 0usize;
    for (gid, title, platform_id, metadata_json) in &games {
        // Gather files
        let mut fstmt = conn.prepare(
            "SELECT path, filename, extension, size, hash, crc32, md5, sha1, sha512, archive_hash FROM game_files gf JOIN game_file_links l ON l.file_id = gf.id WHERE l.game_id = ?1",
        )?;
        let files: Vec<serde_json::Value> = fstmt
            .query_map([gid.clone()], |r| {
                Ok(serde_json::json!({
                    "path": r.get::<_, String>(0)?,
                    "filename": r.get::<_, String>(1)?,
                    "extension": r.get::<_, String>(2)?,
                    "size": r.get::<_, i64>(3)?,
                    "hash": r.get::<_, Option<String>>(4)?,
                    "crc32": r.get::<_, Option<String>>(5)?,
                    "md5": r.get::<_, Option<String>>(6)?,
                    "sha1": r.get::<_, Option<String>>(7)?,
                    "sha512": r.get::<_, Option<String>>(8)?,
                    "archive_hash": r.get::<_, Option<String>>(9)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Write per-game metadata file.
        let game_json = serde_json::json!({
            "id": gid,
            "title": title,
            "platform_id": platform_id,
            "metadata": metadata_json.as_ref().and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok()),
            "files": files,
        });
        let entry_path = format!("games/{}.json", gid);
        zip.start_file(&entry_path, options)
            .map_err(|e| RetroError::Other(e.to_string()))?;
        zip.write_all(game_json.to_string().as_bytes())
            .map_err(RetroError::Io)?;

        manifest_games.push(serde_json::json!({"id": gid, "title": title, "platform_id": platform_id, "file": entry_path}));
        count += 1;
    }

    // Write manifest.
    let manifest = serde_json::json!({
        "version": 1,
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "game_count": count,
        "games": manifest_games,
    });
    zip.start_file("manifest.json", options)
        .map_err(|e| RetroError::Other(e.to_string()))?;
    zip.write_all(manifest.to_string().as_bytes())
        .map_err(RetroError::Io)?;
    zip.finish().map_err(|e| RetroError::Other(e.to_string()))?;
    Ok(count)
}
