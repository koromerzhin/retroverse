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
        let is_bad = if let Some(ref mid) = matched {
            conn.query_row(
                "SELECT bad FROM dat_entries WHERE id=?1",
                [mid.clone()],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0)
                != 0
        } else {
            false
        };
        let status = if matched.is_some() {
            if is_bad { "bad_dump" } else { "verified" }
        } else {
            "unknown"
        };
        if matched.is_some() && !is_bad {
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

#[tauri::command]
pub fn list_audit_log(
    state: State<AppState>,
    limit: Option<usize>,
) -> Result<Vec<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    let lim = limit.unwrap_or(50) as i64;
    let mut stmt = conn.prepare("SELECT id,timestamp,action,target_type,target_id,details,provenance FROM audit_log ORDER BY timestamp DESC LIMIT ?1")?;
    let rows = stmt.query_map([lim], |r| {
        Ok(serde_json::json!({
            "id": r.get::<_, String>(0)?,
            "timestamp": r.get::<_, String>(1)?,
            "action": r.get::<_, String>(2)?,
            "target_type": r.get::<_, Option<String>>(3)?,
            "target_id": r.get::<_, Option<String>>(4)?,
            "details": r.get::<_, Option<String>>(5)?,
            "provenance": r.get::<_, Option<String>>(6)?,
        }))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[tauri::command]
pub fn log_audit(
    state: State<AppState>,
    action: String,
    target_type: Option<String>,
    target_id: Option<String>,
    details: Option<String>,
    provenance: Option<String>,
) -> Result<()> {
    let conn = state.db.lock().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO audit_log (id,timestamp,action,target_type,target_id,details,provenance) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![id, now, action, target_type, target_id, details, provenance],
    )?;
    Ok(())
}

#[allow(clippy::type_complexity)]
#[tauri::command]
pub fn export_dat(state: State<AppState>, dest_path: String) -> Result<usize> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT g.title, g.platform_id, gf.size, gf.crc32, gf.md5, gf.sha1 FROM games g JOIN game_file_links l ON l.game_id=g.id JOIN game_files gf ON gf.id=l.file_id ORDER BY g.title")?;
    let rows: Vec<(
        String,
        Option<String>,
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = stmt
        .query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();
    let mut content = String::new();
    content.push_str("clrmamepro (\n");
    content.push_str("    name \"Retroverse Export\"\n");
    content.push_str("    description \"Exported from RETROVERSE\"\n");
    content.push_str(")\n\n");
    let mut count = 0usize;
    for (title, _platform, size, crc, md5, sha1) in &rows {
        content.push_str("game (\n");
        content.push_str(&format!("    name \"{}\"\n", title.replace('"', "'")));
        content.push_str("    rom (\n");
        content.push_str(&format!("        name \"{}\"\n", title.replace('"', "'")));
        content.push_str(&format!("        size {}\n", size));
        if let Some(c) = crc {
            content.push_str(&format!("        crc {}\n", c));
        }
        if let Some(m) = md5 {
            content.push_str(&format!("        md5 {}\n", m));
        }
        if let Some(s) = sha1 {
            content.push_str(&format!("        sha1 {}\n", s));
        }
        content.push_str("    )\n");
        content.push_str(")\n\n");
        count += 1;
    }
    std::fs::write(&dest_path, content).map_err(RetroError::Io)?;
    Ok(count)
}

#[tauri::command]
pub fn import_preservation_pack(state: State<AppState>, path: String) -> Result<usize> {
    let file = std::fs::File::open(&path).map_err(RetroError::Io)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| RetroError::Other(e.to_string()))?;
    let mut manifest_file = archive
        .by_name("manifest.json")
        .map_err(|e| RetroError::Other(e.to_string()))?;
    use std::io::Read;
    let mut manifest_str = String::new();
    manifest_file
        .read_to_string(&mut manifest_str)
        .map_err(RetroError::Io)?;
    drop(manifest_file);
    let manifest: serde_json::Value =
        serde_json::from_str(&manifest_str).map_err(RetroError::Serde)?;
    let conn = state.db.lock().unwrap();
    let now = chrono::Utc::now().to_rfc3339();
    let mut count = 0usize;
    if let Some(games) = manifest["games"].as_array() {
        for g in games {
            let gid = g["id"].as_str().unwrap_or("").to_string();
            if gid.is_empty() {
                continue;
            }
            let title = g["title"].as_str().unwrap_or("Unknown").to_string();
            let platform_id = g["platform_id"].as_str().map(|s| s.to_string());
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM games WHERE id=?1",
                    [gid.clone()],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap_or(0)
                > 0;
            if !exists {
                conn.execute("INSERT INTO games (id,title,platform_id,created_at,updated_at,metadata_json) VALUES (?1,?2,?3,?4,?5,NULL)", params![gid, title, platform_id, now, now])?;
            } else {
                conn.execute(
                    "UPDATE games SET title=?1, platform_id=?2, updated_at=?3 WHERE id=?4",
                    params![title, platform_id, now, gid],
                )?;
            }
            let game_file = g["file"].as_str().unwrap_or("");
            if !game_file.is_empty()
                && let Ok(mut zf) = archive.by_name(game_file)
            {
                let mut content = String::new();
                let _ = zf.read_to_string(&mut content);
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content)
                    && let Some(meta) = v.get("metadata")
                {
                    let meta_str = meta.to_string();
                    conn.execute(
                        "UPDATE games SET metadata_json=?1 WHERE id=?2",
                        params![meta_str, gid],
                    )?;
                }
            }
            count += 1;
        }
    }
    Ok(count)
}
