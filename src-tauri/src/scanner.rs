use crate::error::{Result, RetroError};
use crate::identify;
use crate::models::{GameFile, ScanResult};
use chrono::Utc;
use rusqlite::{Connection, params};
use std::path::Path;
use walkdir::WalkDir;

pub fn scan_source(conn: &mut Connection, source_id: &str, app: Option<tauri::AppHandle>) -> Result<ScanResult> {
    use tauri::Emitter;
    let (path,): (String,) = conn
        .query_row(
            "SELECT path FROM game_sources WHERE id = ?1",
            [source_id],
            |r| Ok((r.get(0)?,)),
        )
        .map_err(|_| RetroError::NotFound("source".into()))?;

    let root = Path::new(&path);
    if !root.exists() {
        return Err(RetroError::Invalid(format!(
            "path does not exist: {}",
            path
        )));
    }

    let platforms = identify::load_platforms(conn)?;

    // Collect files
    let mut files: Vec<(String, u64, i64)> = Vec::new(); // path, size, mtime
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        let pstr = p.to_string_lossy();
        // Skip media/asset directories to avoid scanning thousands of images.
        // Match whole path segments only, but not the /run/media mount point.
        let segs: Vec<&str> = pstr.split('/').collect();
        let skip = segs.iter().enumerate().any(|(i, seg)| {
            (*seg == "media" || *seg == "images" || *seg == "assets")
                && i > 0
                && segs[i - 1] != "run"
        });
        if skip {
            continue;
        }
        let ext = identify::ext_of(pstr.as_ref());
        if ext.is_empty() {
            continue;
        }
        // Consider files whose extension maps to a known platform OR whose path
        // identifies a platform (e.g. .zip ROMs in a nes/ folder).
        let ext_known = identify::guess_platform(&ext, &platforms).is_some();
        let path_known = identify::infer_platform_from_path(p.to_string_lossy().as_ref(), &platforms).is_some();
        if !ext_known && !path_known {
            continue;
        }
        // only known-ish extensions: accept all but skip tiny/system? We'll accept all files with extension
        let meta = match std::fs::metadata(p) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        files.push((p.to_string_lossy().to_string(), meta.len(), mtime));
    }

    let _total = files.len();
    let mut files_found = 0usize;
    let mut games_created = 0usize;
    let mut games_updated = 0usize;

    // Process each file in a transaction
    let tx = conn.transaction()?;
    let total = files.len();
    if let Some(a) = &app {
        let _ = a.emit("scan-progress", serde_json::json!({"source_id": source_id, "scanned": 0usize, "total": total, "status": "scanning"}));
    }
    for (idx, (fpath, size, mtime)) in files.iter().enumerate() {
        let filename = Path::new(fpath)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let ext = identify::ext_of(fpath);
        let platform_id = identify::infer_platform_from_path(fpath, &platforms)
            .or_else(|| identify::guess_platform(&ext, &platforms));

        // upsert game_file
        let existing: Option<String> = tx
            .query_row(
                "SELECT id FROM game_files WHERE source_id=?1 AND path=?2",
                params![source_id, fpath],
                |r| r.get(0),
            )
            .ok();

        let file_id = if let Some(id) = existing {
            tx.execute(
                "UPDATE game_files SET filename=?1,extension=?2,size=?3,modified_at=?4,platform_id=?5 WHERE id=?6",
                params![filename, ext, *size as i64, *mtime, platform_id, id],
            )?;
            id
        } else {
            let id = crate::models::new_id();
            tx.execute(
                "INSERT INTO game_files (id,source_id,path,filename,extension,size,modified_at,platform_id,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![id, source_id, fpath, filename, ext, *size as i64, *mtime, platform_id, Utc::now().to_rfc3339()],
            )?;
            id
        };
        files_found += 1;

        // identify game title
        let gf = GameFile {
            id: file_id.clone(),
            source_id: source_id.to_string(),
            path: fpath.clone(),
            filename: filename.clone(),
            extension: ext.clone(),
            size: *size as i64,
            modified_at: *mtime,
            hash: None,
            platform_id: platform_id.clone(),
            created_at: Utc::now(),
        };
        let (title, conf) = identify::identify_file(&gf, &platforms);

        // find existing game by title+platform (case-insensitive)
        let existing_game: Option<String> = tx
            .query_row(
                "SELECT id FROM games WHERE LOWER(title)=LOWER(?1) AND (platform_id IS ?2 OR platform_id=?2)",
                params![title, platform_id],
                |r| r.get(0),
            )
            .ok();

        let game_id = if let Some(gid) = existing_game {
            tx.execute(
                "UPDATE games SET updated_at=?1 WHERE id=?2",
                params![Utc::now().to_rfc3339(), gid],
            )?;
            games_updated += 1;
            gid
        } else {
            let gid = crate::models::new_id();
            tx.execute(
                "INSERT INTO games (id,title,platform_id,created_at,updated_at) VALUES (?1,?2,?3,?4,?5)",
                params![gid, title, platform_id, Utc::now().to_rfc3339(), Utc::now().to_rfc3339()],
            )?;
            games_created += 1;
            gid
        };

        // link file->game if not manual override exists
        let manual: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM game_file_links l JOIN user_corrections c ON c.file_id=l.file_id WHERE l.file_id=?1",
                [file_id.clone()],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if manual == 0 {
            tx.execute(
                "INSERT OR REPLACE INTO game_file_links (file_id,game_id,confidence,manual) VALUES (?1,?2,?3,0)",
                params![file_id, game_id, conf],
            )?;
        }
        if let Some(a) = &app
            && (idx.is_multiple_of(50) || idx + 1 == total)
        {
            let _ = a.emit("scan-progress", serde_json::json!({"source_id": source_id, "scanned": idx + 1, "total": total, "status": "scanning"}));
        }
    }

    // update source last_scan_at
    tx.execute(
        "UPDATE game_sources SET last_scan_at=?1 WHERE id=?2",
        params![Utc::now().to_rfc3339(), source_id],
    )?;

    // record sync job
    let job_id = crate::models::new_id();
    tx.execute(
        "INSERT INTO sync_jobs (id,source_id,status,started_at,finished_at,files_found) VALUES (?1,?2,?3,?4,?5,?6)",
        params![job_id, source_id, "completed", Utc::now().to_rfc3339(), Utc::now().to_rfc3339(), files_found as i64],
    )?;

    tx.commit()?;

    if let Some(a) = &app {
        let _ = a.emit("scan-progress", serde_json::json!({"source_id": source_id, "scanned": total, "total": total, "status": "done"}));
    }

    Ok(ScanResult {
        source_id: source_id.to_string(),
        files_found,
        games_created,
        games_updated,
    })
}
