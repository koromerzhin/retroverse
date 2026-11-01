use crate::error::{Result, RetroError};
use crate::identify;
use crate::models::{GameFile, ScanResult};
use chrono::Utc;
use rusqlite::Connection;
use std::path::Path;
use walkdir::WalkDir;

pub fn scan_source(
    conn: &mut Connection,
    source_id: &str,
    app: Option<tauri::AppHandle>,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    folder_override: Option<String>,
) -> Result<ScanResult> {
    use tauri::Emitter;
    tracing::info!("scan_source start source_id={} folder={:?}", source_id, folder_override);

    let path = if let Some(f) = folder_override {
        f
    } else {
        conn.query_row(
            "SELECT path FROM game_sources WHERE id = ?1",
            [source_id],
            |r| r.get(0),
        )
        .map_err(|_| RetroError::NotFound("source".into()))?
    };

    let mut root = std::path::Path::new(&path).to_path_buf();
    if !root.exists() {
        // Source inaccessible: do nothing, don't mark missing.
        if let Some(a) = &app {
            let _ = a.emit(
                "scan-progress",
                serde_json::json!({"source_id": source_id, "scanned": 0, "total": 0, "status": "skipped", "reason": "inaccessible"}),
            );
        }
        return Ok(ScanResult {
            source_id: source_id.to_string(),
            files_found: 0,
            games_created: 0,
            games_updated: 0,
        });
    }
    // If a 'roms' subfolder exists, scan it as the primary source.
    if !root.ends_with("roms") {
        let roms = root.join("roms");
        if roms.is_dir() {
            root = roms;
        }
    }
    // Load systems.txt if present to enrich platform list.
    let systems_path = root.join("systems.txt");
    if systems_path.is_file()
        && let Ok(content) = std::fs::read_to_string(&systems_path)
    {
        let systems = identify::parse_systems_txt(&content);
        let _ = identify::ensure_platforms(conn, &systems);
    }

    let platforms = identify::load_platforms(conn)?;

    // Determine top-level entries to scan incrementally.
    let mut top_dirs: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&root) {
        for e in rd.flatten() {
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let name = e.file_name().to_string_lossy().to_string();
                // Skip media/asset dirs
                if name == "media" || name == "images" || name == "assets" {
                    continue;
                }
                top_dirs.push(e.path());
            }
        }
    }
    // If no subdirs, scan root itself.
    if top_dirs.is_empty() {
        top_dirs.push(root.clone());
    }

    let mut total_files_found = 0usize;
    let mut total_created = 0usize;
    let mut total_updated = 0usize;

    for top in &top_dirs {
        if cancel.load(std::sync::atomic::Ordering::SeqCst) {
            if let Some(a) = &app {
                use tauri::Emitter;
                let _ = a.emit("scan-progress", serde_json::json!({"source_id": source_id, "scanned": total_files_found, "total": total_files_found, "status": "cancelled"}));
            }
            return Ok(ScanResult {
                source_id: source_id.to_string(),
                files_found: total_files_found,
                games_created: total_created,
                games_updated: total_updated,
            });
        }
        // Collect files in this top-level folder.
        // Determine allowed extensions from systeminfo.txt if present.
        let allowed_exts: Vec<String> = {
            let si = top.join("systeminfo.txt");
            if si.is_file() {
                if let Ok(content) = std::fs::read_to_string(&si) {
                    identify::parse_systeminfo_txt(&content).unwrap_or_default()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        };
        // Load gamelist for this platform folder (if any) to know already-processed games.
        let gamelist_known: std::collections::HashSet<String> = {
            let slug = top.file_name().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            let emulation_root = top.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf());
            if let Some(er) = emulation_root {
                let gp = er.join("saves").join("es-de").join("gamelists").join(&slug).join("gamelist.xml");
                if gp.is_file() {
                    if let Ok(content) = std::fs::read_to_string(&gp) {
                        identify::parse_gamelist_paths(&content)
                    } else {
                        std::collections::HashSet::new()
                    }
                } else {
                    std::collections::HashSet::new()
                }
            } else {
                std::collections::HashSet::new()
            }
        };

        let mut files: Vec<(String, u64, i64, Vec<String>)> = Vec::new();
        for entry in WalkDir::new(top)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if cancel.load(std::sync::atomic::Ordering::SeqCst) {
                // cancelled during walk
                if let Some(a) = &app {
                    use tauri::Emitter;
                    let _ = a.emit("scan-progress", serde_json::json!({"source_id": source_id, "scanned": total_files_found, "total": total_files_found, "status": "cancelled"}));
                }
                return Ok(ScanResult {
                    source_id: source_id.to_string(),
                    files_found: total_files_found,
                    games_created: total_created,
                    games_updated: total_updated,
                });
            }
            if !entry.file_type().is_file() {
                continue;
            }
            let p = entry.path();
            let pstr = p.to_string_lossy();
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
            let ext_ok = if !allowed_exts.is_empty() {
                allowed_exts.iter().any(|a| a.eq_ignore_ascii_case(&format!(".{}", ext)))
            } else {
                identify::guess_platform(&ext, &platforms).is_some()
                    || identify::infer_platform_from_path(pstr.as_ref(), &platforms).is_some()
            };
            if !ext_ok {
                continue;
            }
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
            files.push((p.to_string_lossy().to_string(), meta.len(), mtime, allowed_exts.clone()));
        }

        // Emit progress start for this folder.
        if let Some(a) = &app {
            let _ = a.emit(
                "scan-progress",
                serde_json::json!({"source_id": source_id, "scanned": 0usize, "total": files.len(), "status": "scanning"}),
            );
        }

        // Process this folder in its own transaction.
        let mut files_found = 0usize;
        let mut games_created = 0usize;
        let mut games_updated = 0usize;
        let tx = conn.transaction()?;
        // collect found paths for missing detection
        let mut found_paths: Vec<String> = Vec::new();
        for (idx, (fpath, size, mtime, allowed)) in files.iter().enumerate() {
            if cancel.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            let filename = Path::new(fpath)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let ext = identify::ext_of(fpath);
            let platform_id = identify::infer_platform_from_path(fpath, &platforms)
                .or_else(|| identify::guess_platform(&ext, &platforms));

            let existing: Option<(String, i64, i64, bool)> = tx
                .query_row(
                    "SELECT id,size,modified_at, (hash IS NULL OR crc32 IS NULL OR md5 IS NULL OR sha1 IS NULL) FROM game_files WHERE source_id=?1 AND path=?2",
                    rusqlite::params![source_id, fpath],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get::<_, i64>(3)? != 0)),
                )
                .ok();

            // Incremental: skip unchanged files only if hashes already present.
            if let Some((_id, old_size, old_mtime, needs_hash)) = &existing
                && *old_size == *size as i64
                && *old_mtime == *mtime
                && !needs_hash
            {
                found_paths.push(fpath.clone());
                files_found += 1;
                // still emit progress occasionally
                if let Some(a) = &app
                    && (idx.is_multiple_of(200) || idx + 1 == files.len())
                {
                    let _ = a.emit(
                        "scan-progress",
                        serde_json::json!({"source_id": source_id, "scanned": idx + 1, "total": files.len(), "status": "scanning", "file": filename}),
                    );
                }
                continue;
            }

            let (sha256, crc32, md5, sha1, zip_inner) = identify::compute_hashes(fpath, allowed);

            let file_id = if let Some((id, _, _, _)) = existing {
                tx.execute(
                    "UPDATE game_files SET filename=?1,extension=?2,size=?3,modified_at=?4,platform_id=?5,hash=?6,crc32=?7,md5=?8,sha1=?9,zip_inner=?10,missing=0 WHERE id=?11",
                    rusqlite::params![filename, ext, *size as i64, *mtime, platform_id, sha256, crc32, md5, sha1, zip_inner, id],
                )?;
                id
            } else {
                let id = crate::models::new_id();
                tx.execute(
                    "INSERT INTO game_files (id,source_id,path,filename,extension,size,modified_at,platform_id,hash,crc32,md5,sha1,zip_inner,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
                    rusqlite::params![id, source_id, fpath, filename, ext, *size as i64, *mtime, platform_id, sha256, crc32, md5, sha1, zip_inner, Utc::now().to_rfc3339()],
                )?;
                id
            };
            files_found += 1;
            found_paths.push(fpath.clone());

            // If this game is already in ES-DE gamelist, skip re-processing.
            if gamelist_known.contains(&filename.to_lowercase()) {
                // still emit progress
                if let Some(a) = &app
                    && (idx.is_multiple_of(200) || idx + 1 == files.len())
                {
                    let _ = a.emit(
                        "scan-progress",
                        serde_json::json!({"source_id": source_id, "scanned": idx + 1, "total": files.len(), "status": "scanning", "file": filename}),
                    );
                }
                continue;
            }

            let gf = GameFile {
                id: file_id.clone(),
                source_id: source_id.to_string(),
                path: fpath.clone(),
                filename: filename.clone(),
                extension: ext.clone(),
                size: *size as i64,
                modified_at: *mtime,
                hash: sha256.clone(),
                crc32: crc32.clone(),
                md5: md5.clone(),
                sha1: sha1.clone(),
                zip_inner: zip_inner.clone(),
                platform_id: platform_id.clone(),
                created_at: Utc::now(),
            };
            let (title, conf) = identify::identify_file(&gf, &platforms);

            let existing_game: Option<String> = tx
                .query_row(
                    "SELECT id FROM games WHERE LOWER(title)=LOWER(?1) AND (platform_id IS ?2 OR platform_id=?2)",
                    rusqlite::params![title, platform_id],
                    |r| r.get(0),
                )
                .ok();

            let game_id = if let Some(gid) = existing_game {
                tx.execute(
                    "UPDATE games SET updated_at=?1 WHERE id=?2",
                    rusqlite::params![Utc::now().to_rfc3339(), gid],
                )?;
                games_updated += 1;
                gid
            } else {
                let gid = crate::models::new_id();
                tx.execute(
                    "INSERT INTO games (id,title,platform_id,created_at,updated_at) VALUES (?1,?2,?3,?4,?5)",
                    rusqlite::params![gid, title, platform_id, Utc::now().to_rfc3339(), Utc::now().to_rfc3339()],
                )?;
                games_created += 1;
                gid
            };

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
                    rusqlite::params![file_id, game_id, conf],
                )?;
            }

            if let Some(a) = &app
                && (idx.is_multiple_of(50) || idx + 1 == files.len())
            {
                let _ = a.emit(
                    "scan-progress",
                    serde_json::json!({"source_id": source_id, "scanned": idx + 1, "total": files.len(), "status": "scanning"}),
                );
            }
        }

        // Mark missing files in this folder (only if accessible, which it is).
        // Mark all files of this source under this top folder as missing, then unmark found.
        // Simpler: mark all source files missing, then unmark all found_paths.
        // But that would affect other folders not yet scanned. Instead, mark only files
        // whose path starts with this top folder.
        let top_str = top.to_string_lossy().to_string();
        tx.execute(
            "UPDATE game_files SET missing=1 WHERE source_id=?1 AND path LIKE ?2",
            rusqlite::params![source_id, format!("{}%", top_str)],
        )?;
        for fp in &found_paths {
            tx.execute(
                "UPDATE game_files SET missing=0 WHERE source_id=?1 AND path=?2",
                rusqlite::params![source_id, fp],
            )?;
        }

        tx.commit()?;

        total_files_found += files_found;
        total_created += games_created;
        total_updated += games_updated;

        if let Some(a) = &app {
            let _ = a.emit(
                "scan-progress",
                serde_json::json!({"source_id": source_id, "scanned": files_found, "total": files_found, "status": "folder_done"}),
            );
        }
    }

    // Update source last_scan_at and availability.
    conn.execute(
        "UPDATE game_sources SET last_scan_at=?1, available=1, last_error=NULL WHERE id=?2",
        rusqlite::params![Utc::now().to_rfc3339(), source_id],
    )?;

    // Record sync job.
    let job_id = crate::models::new_id();
    conn.execute(
        "INSERT INTO sync_jobs (id,source_id,status,started_at,finished_at,files_found) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params![job_id, source_id, "completed", Utc::now().to_rfc3339(), Utc::now().to_rfc3339(), total_files_found as i64],
    )?;

    // Clean up orphan games (no linked files).
    let _ = crate::db::cleanup_orphan_games(conn);

    if let Some(a) = &app {
        let _ = a.emit(
            "scan-progress",
            serde_json::json!({"source_id": source_id, "scanned": total_files_found, "total": total_files_found, "status": "done", "files_found": total_files_found, "games_created": total_created, "games_updated": total_updated}),
        );
    }

    tracing::info!(
        "scan_source done source_id={} files={} created={} updated={}",
        source_id, total_files_found, total_created, total_updated
    );
    Ok(ScanResult {
        source_id: source_id.to_string(),
        files_found: total_files_found,
        games_created: total_created,
        games_updated: total_updated,
    })
}