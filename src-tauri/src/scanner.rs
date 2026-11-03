use crate::error::{Result, RetroError};
use crate::identify;
use crate::models::{GameFile, ScanResult};
use crate::state::AppState;
use chrono::Utc;

use std::path::Path;
use walkdir::WalkDir;

pub fn scan_source(
    state: &AppState,
    source_id: &str,
    app: Option<tauri::AppHandle>,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    folder_override: Option<String>,
    platform_filter: Option<String>,
) -> Result<ScanResult> {
    use tauri::Emitter;

    if let Some(a) = &app {
        let _ = a.emit(
            "scan-progress",
            serde_json::json!({"source_id": source_id, "scanned": 0, "total": 0, "status": "scanning"}),
        );
    }

    let path = if let Some(f) = folder_override {
        f
    } else {
        let conn = state.db.lock().unwrap();
        conn.query_row(
            "SELECT path FROM game_sources WHERE id = ?1",
            [source_id],
            |r| r.get(0),
        )
        .map_err(|_| RetroError::NotFound("source".into()))?
    };

    let mut root = std::path::Path::new(&path).to_path_buf();
    if !root.exists() {
        if let Some(a) = &app {
            let _ = a.emit(
                "scan-progress",
                serde_json::json!({"source_id": source_id, "scanned": 0, "total": 0, "status": "skipped"}),
            );
        }
        return Ok(ScanResult {
            source_id: source_id.to_string(),
            files_found: 0,
            games_created: 0,
            games_updated: 0,
        });
    }

    // (No folder-level skip: file-level incremental skip handles already-present files.)
    if !root.ends_with("roms") {
        let roms = root.join("roms");
        if roms.is_dir() {
            root = roms;
        }
    }
    let current_mtime = std::fs::metadata(&root)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let systems_path = root.join("systems.txt");
    if systems_path.is_file()
        && let Ok(content) = std::fs::read_to_string(&systems_path)
    {
        let systems = identify::parse_systems_txt(&content);
        let conn = state.db.lock().unwrap();
        let _ = identify::ensure_platforms(&conn, &systems);
    }

    let platforms = {
        let conn = state.db.lock().unwrap();
        identify::load_platforms(&conn)?
    };

    let mut top_dirs: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&root) {
        for e in rd.flatten() {
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let name = e.file_name().to_string_lossy().to_string();
                if name == "media" || name == "images" || name == "assets" {
                    continue;
                }
                top_dirs.push(e.path());
            }
        }
    }
    if top_dirs.is_empty() {
        top_dirs.push(root.clone());
    }

    let mut total_files_found = 0usize;
    let mut total_created = 0usize;
    let mut total_updated = 0usize;

    for top in &top_dirs {
        if cancel.load(std::sync::atomic::Ordering::SeqCst) {
            if let Some(a) = &app {
                let _ = a.emit(
                    "scan-progress",
                    serde_json::json!({"source_id": source_id, "scanned": total_files_found, "total": total_files_found, "status": "cancelled"}),
                );
            }
            return Ok(ScanResult {
                source_id: source_id.to_string(),
                files_found: total_files_found,
                games_created: total_created,
                games_updated: total_updated,
            });
        }

        let gamelist_known: std::collections::HashSet<String> = {
            let slug = top
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            let emulation_root = top
                .parent()
                .and_then(|p| p.parent())
                .map(|p| p.to_path_buf());
            if let Some(er) = emulation_root {
                let gp = er
                    .join("saves")
                    .join("es-de")
                    .join("gamelists")
                    .join(&slug)
                    .join("gamelist.xml");
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

        let mut files: Vec<(String, u64, i64, Vec<String>)> = Vec::new();
        for entry in WalkDir::new(top)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
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
            if p.file_name().and_then(|n| n.to_str()) == Some("systeminfo.txt") {
                continue;
            }
            let ext = identify::ext_of(pstr.as_ref());
            let ext_ok = if !allowed_exts.is_empty() {
                allowed_exts
                    .iter()
                    .any(|a| a.eq_ignore_ascii_case(&format!(".{}", ext)))
            } else {
                identify::guess_platform(&ext, &platforms).is_some()
                    || identify::infer_platform_from_path(pstr.as_ref(), &platforms).is_some()
            };
            if !ext_ok {
                continue;
            }
            // Platform filter: only include files under a folder matching the platform slug.
            if let Some(pf) = &platform_filter {
                let low = pf.to_lowercase();
                if !pstr.to_lowercase().contains(&format!("/{}/", low)) {
                    continue;
                }
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
            files.push((
                p.to_string_lossy().to_string(),
                meta.len(),
                mtime,
                allowed_exts.clone(),
            ));
        }

        if let Some(a) = &app {
            let _ = a.emit(
                "scan-progress",
                serde_json::json!({"source_id": source_id, "scanned": 0usize, "total": files.len(), "status": "scanning", "folder": top.to_string_lossy()}),
            );
        }

        // Process this folder in its own transaction (lock acquired/released per folder).
        let mut files_found = 0usize;
        let mut games_created = 0usize;
        let mut games_updated = 0usize;
        let mut found_paths: Vec<String> = Vec::new();
        {
            let mut conn = state.db.lock().unwrap();
            let tx = conn.transaction()?;
            for (idx, (fpath, size, mtime, _allowed)) in files.iter().enumerate() {
                if cancel.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                let filename = Path::new(fpath)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let ext = identify::ext_of(fpath);
                let platform_id = if let Some(pf) = &platform_filter {
                    Some(pf.clone())
                } else {
                    identify::infer_platform_from_path(fpath, &platforms)
                        .or_else(|| identify::guess_platform(&ext, &platforms))
                };

                if ext == "scummvm" {
                    let code = std::path::Path::new(&filename)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(&filename);
                    if let Some(new_title) = identify::scummvm_title(code) {
                        let _ = tx.execute(
                            "UPDATE games SET title=?1 WHERE LOWER(title)=LOWER(?2) AND (platform_id IS ?3 OR platform_id=?3)",
                            rusqlite::params![new_title, code, platform_id],
                        );
                    }
                }

                let existing: Option<(String, i64, i64, Option<String>)> = tx
                    .query_row(
                        "SELECT id,size,modified_at,platform_id FROM game_files WHERE source_id=?1 AND path=?2",
                        rusqlite::params![source_id, fpath],
                        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                    )
                    .ok();

                // Reprocess if platform_id differs from inferred.
                let platform_changed = if let Some((_id, _, _, stored)) = &existing {
                    stored.as_ref() != platform_id.as_ref()
                } else {
                    false
                };
                if let Some((_id, old_size, old_mtime, _)) = &existing
                    && *old_size == *size as i64
                    && *old_mtime == *mtime
                    && !platform_changed
                {
                    found_paths.push(fpath.clone());
                    files_found += 1;
                    if let Some(a) = &app
                        && (idx.is_multiple_of(200) || idx + 1 == files.len())
                    {
                        let _ = a.emit(
                            "scan-progress",
                            serde_json::json!({"source_id": source_id, "scanned": idx + 1, "total": files.len(), "status": "scanning", "folder": top.to_string_lossy(), "file": filename}),
                        );
                    }
                    continue;
                }

                // Hashes are computed on-demand via "Calculer les empreintes" (scan stays fast).
                let (sha256, crc32, md5, sha1, zip_inner, sha512) =
                    (None, None, None, None, None, None);

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

                if gamelist_known.contains(&filename.to_lowercase()) {
                    if let Some(a) = &app
                        && (idx.is_multiple_of(200) || idx + 1 == files.len())
                    {
                        let _ = a.emit(
                            "scan-progress",
                            serde_json::json!({"source_id": source_id, "scanned": idx + 1, "total": files.len(), "status": "scanning", "folder": top.to_string_lossy(), "file": filename}),
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
                    sha512: sha512.clone(),
                    zip_inner: zip_inner.clone(),
                    archive_hash: None,
                    archive_crc32: None,
                    archive_md5: None,
                    archive_sha1: None,
                    archive_sha512: None,
                    platform_id: platform_id.clone(),
                    created_at: Utc::now(),
                    is_dir: false,
                };
                let (mut title, conf) = identify::identify_file(&gf, &platforms);
                if ext == "scummvm" {
                    let code = std::fs::read_to_string(fpath)
                        .map(|c| c.trim().to_string())
                        .unwrap_or_else(|_| {
                            std::path::Path::new(&filename)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or(&filename)
                                .to_string()
                        });
                    if let Some(t) = identify::scummvm_title(&code) {
                        title = t;
                    }
                }

                let existing_game: Option<String> = tx
                    .query_row(
                        "SELECT id FROM games WHERE LOWER(title)=LOWER(?1) AND (platform_id IS ?2 OR platform_id=?2)",
                        rusqlite::params![title, platform_id],
                        |r| r.get(0),
                    )
                    .ok();

                let game_id = if let Some(gid) = existing_game {
                    // Update the title if it changed (e.g. ScummVM resolution) and no user correction exists.
                    let has_correction: i64 = tx
                        .query_row(
                            "SELECT COUNT(*) FROM user_corrections c JOIN game_file_links l ON l.file_id = c.file_id WHERE l.game_id = ?1",
                            [gid.clone()],
                            |r| r.get(0),
                        )
                        .unwrap_or(0);
                    if has_correction == 0 {
                        tx.execute(
                            "UPDATE games SET title=?1, updated_at=?2 WHERE id=?3",
                            rusqlite::params![title, Utc::now().to_rfc3339(), gid],
                        )?;
                    } else {
                        tx.execute(
                            "UPDATE games SET updated_at=?1 WHERE id=?2",
                            rusqlite::params![Utc::now().to_rfc3339(), gid],
                        )?;
                    }
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
                    && (idx.is_multiple_of(200) || idx + 1 == files.len())
                {
                    let _ = a.emit(
                        "scan-progress",
                        serde_json::json!({"source_id": source_id, "scanned": idx + 1, "total": files.len(), "status": "scanning", "folder": top.to_string_lossy(), "file": filename}),
                    );
                }
            }
            if platform_filter.is_none() {
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
            }

            tx.commit()?;
        } // close transaction block
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

    let conn = state.db.lock().unwrap();
    if platform_filter.is_none() {
        // Remove files that disappeared since the last scan.
        let _ = conn.execute(
            "DELETE FROM game_files WHERE source_id=?1 AND missing=1",
            [source_id],
        );
        let _ = crate::db::cleanup_orphan_games(&conn);
    }
    conn.execute(
        "UPDATE game_sources SET last_scan_at=?1, available=1, last_error=NULL, root_mtime=?2 WHERE id=?3",
        rusqlite::params![Utc::now().to_rfc3339(), current_mtime, source_id],
    )?;

    let job_id = crate::models::new_id();
    conn.execute(
        "INSERT INTO sync_jobs (id,source_id,status,started_at,finished_at,files_found) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params![job_id, source_id, "completed", Utc::now().to_rfc3339(), Utc::now().to_rfc3339(), total_files_found as i64],
    )?;

    if let Some(a) = &app {
        let _ = a.emit(
            "scan-progress",
            serde_json::json!({"source_id": source_id, "scanned": total_files_found, "total": total_files_found, "status": "done", "files_found": total_files_found, "games_created": total_created, "games_updated": total_updated}),
        );
    }

    Ok(ScanResult {
        source_id: source_id.to_string(),
        files_found: total_files_found,
        games_created: total_created,
        games_updated: total_updated,
    })
}
