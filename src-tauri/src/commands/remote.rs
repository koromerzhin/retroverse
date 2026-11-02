use crate::error::{Result, RetroError};
use crate::models::{GameSource, RemoteEntry, ScanResult, TransferResult};
use crate::remote;
use crate::state::AppState;
use chrono::Utc;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub async fn test_remote_connection(
    host: String,
    port: u16,
    username: String,
    key_path: String,
) -> Result<String> {
    let sess = remote::connect(&host, port, &username, &key_path).await?;
    // get current dir
    let cwd = sess
        .sftp
        .canonicalize(".")
        .await
        .unwrap_or_else(|_| "/".into());
    // close
    let _ = sess
        .session
        .disconnect(russh::Disconnect::ByApplication, "", "")
        .await;
    Ok(format!("OK — {}", cwd))
}

#[tauri::command]
pub async fn add_remote_source(
    state: State<'_, AppState>,
    name: String,
    host: String,
    port: u16,
    username: String,
    key_path: String,
    remote_path: String,
) -> Result<GameSource> {
    // test connection first (key path may be empty -> default key)
    let sess = remote::connect(&host, port, &username, &key_path).await?;
    // If a 'roms' subfolder exists, use it as the source path.
    let mut remote_path = remote_path.clone();
    if !remote_path.ends_with("roms")
        && !remote_path.ends_with("roms/")
        && let Ok(rd) = sess.sftp.read_dir(&remote_path).await
    {
        let entries: Vec<_> = rd.collect();
        if entries
            .iter()
            .any(|e| e.file_name() == "roms" && e.metadata().is_dir())
        {
            if remote_path.ends_with('/') {
                remote_path = format!("{}roms/", remote_path);
            } else {
                remote_path = format!("{}/roms", remote_path);
            }
        }
    }
    let _ = sess
        .session
        .disconnect(russh::Disconnect::ByApplication, "", "")
        .await;

    let conn_id = crate::models::new_id();
    let now = Utc::now();
    {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO remote_connections (id,name,host,port,username,key_path,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![conn_id, name, host, port as i64, username, key_path, now.to_rfc3339()],
        )?;
        let src_id = crate::models::new_id();
        conn.execute(
            "INSERT INTO game_sources (id,name,kind,path,created_at,connection_id,remote_path,available) VALUES (?1,?2,?3,?4,?5,?6,?7,1)",
            params![src_id, name.clone(), "remote", remote_path.clone(), now.to_rfc3339(), conn_id, remote_path.clone()],
        )?;
        // store src_id in a variable? need return. We'll re-query? Instead build below.
        // We'll just use src_id local; but need return GameSource. Save to outer via closure? Use a mutable.
        // Simpler: store in a temp table? No. Let's restructure: collect id.
        // We'll push id into a vec in outer scope by making this block return it.
    }
    // Re-query the created source by connection_id+remote_path (latest)
    let src = {
        let conn = state.db.lock().unwrap();
        conn.query_row(
            "SELECT id,name,kind,path,created_at,last_scan_at,connection_id,remote_path,available,last_error FROM game_sources WHERE connection_id=?1 AND remote_path=?2 ORDER BY created_at DESC LIMIT 1",
            params![conn_id, remote_path],
            |r| {
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
            },
        )?
    };
    Ok(src)
}

#[tauri::command]
pub async fn list_remote_dir(
    state: State<'_, AppState>,
    source_id: String,
    path: Option<String>,
) -> Result<Vec<RemoteEntry>> {
    let (host, port, username, key_path, base) = {
        let conn = state.db.lock().unwrap();
        let (cid, rp): (Option<String>, Option<String>) = conn.query_row(
            "SELECT connection_id, remote_path FROM game_sources WHERE id=?1",
            [source_id.clone()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let cid = cid.ok_or_else(|| RetroError::Invalid("not remote".into()))?;
        let (host, port, username, key_path): (String, i64, String, String) = conn.query_row(
            "SELECT host,port,username,key_path FROM remote_connections WHERE id=?1",
            [cid],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?;
        (
            host,
            port as u16,
            username,
            key_path,
            rp.unwrap_or_else(|| "/".into()),
        )
    };

    let target = path.unwrap_or(base);
    let sess = remote::connect(&host, port, &username, &key_path).await?;
    let mut out = Vec::new();
    match sess.sftp.read_dir(&target).await {
        Ok(entries) => {
            for e in entries {
                let meta = e.metadata();
                let name = e.file_name();
                if name == "." || name == ".." {
                    continue;
                }
                let full = if target.ends_with('/') {
                    format!("{}{}", target, name)
                } else {
                    format!("{}/{}", target, name)
                };
                let ext = std::path::Path::new(&name)
                    .extension()
                    .and_then(|x| x.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                out.push(RemoteEntry {
                    name,
                    path: full,
                    is_dir: meta.is_dir(),
                    size: meta.size.unwrap_or(0) as i64,
                    modified_at: meta.mtime.unwrap_or(0) as i64,
                    extension: ext,
                });
            }
        }
        Err(e) => {
            let _ = sess
                .session
                .disconnect(russh::Disconnect::ByApplication, "", "")
                .await;
            return Err(RetroError::Other(e.to_string()));
        }
    }
    let _ = sess
        .session
        .disconnect(russh::Disconnect::ByApplication, "", "")
        .await;
    Ok(out)
}

pub async fn scan_remote_source_inner(
    state: &AppState,
    source_id: String,
    app: Option<tauri::AppHandle>,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    folder_override: Option<String>,
    platform_filter: Option<String>,
) -> Result<ScanResult> {
    tracing::info!(
        "scan_remote_source start source_id={} folder={:?}",
        source_id,
        folder_override
    );
    if let Some(a) = &app {
        use tauri::Emitter;
        let _ = a.emit("scan-progress", serde_json::json!({"source_id": source_id, "scanned": 0, "total": 0, "status": "scanning"}));
    }
    // gather connection info
    let (host, port, username, key_path, mut base_path) = {
        let conn = state.db.lock().unwrap();
        let (cid, rp): (Option<String>, Option<String>) = conn.query_row(
            "SELECT connection_id, remote_path FROM game_sources WHERE id=?1",
            [source_id.clone()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let cid = cid.ok_or_else(|| RetroError::Invalid("not remote".into()))?;
        let (host, port, username, key_path): (String, i64, String, String) = conn.query_row(
            "SELECT host,port,username,key_path FROM remote_connections WHERE id=?1",
            [cid],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?;
        (
            host,
            port as u16,
            username,
            key_path,
            rp.unwrap_or_else(|| "/".into()),
        )
    };

    if let Some(f) = folder_override {
        base_path = f;
    }

    let sess = remote::connect(&host, port, &username, &key_path).await?;

    // If a 'roms' subfolder exists, scan it as the primary source.
    if !base_path.ends_with("roms")
        && !base_path.ends_with("roms/")
        && let Ok(rd) = sess.sftp.read_dir(&base_path).await
    {
        let entries: Vec<_> = rd.collect();
        if entries
            .iter()
            .any(|e| e.file_name() == "roms" && e.metadata().is_dir())
        {
            if base_path.ends_with('/') {
                base_path = format!("{}roms/", base_path);
            } else {
                base_path = format!("{}/roms", base_path);
            }
        }
    }

    // Compute root mtime for tracking (but don't skip the whole scan; file-level incremental handles already-present files).
    let current_mtime: i64 = if let Ok(meta) = sess.sftp.metadata(&base_path).await {
        meta.mtime.unwrap_or(0) as i64
    } else {
        0
    };

    // recursive collect files via sftp
    let mut collected: Vec<(String, i64, i64, Vec<String>)> = Vec::new(); // path, size, mtime, allowed_exts
    #[allow(clippy::too_many_arguments)]
    async fn walk(
        sftp: &russh_sftp::client::SftpSession,
        dir: &str,
        platforms_ext: &[String],
        platform_slugs: &[String],
        allowed_exts: Vec<String>,
        out: &mut Vec<(String, i64, i64, Vec<String>)>,
        app: Option<&tauri::AppHandle>,
        source_id: &str,
        cancel: Option<&std::sync::Arc<std::sync::atomic::AtomicBool>>,
        platform_filter: Option<&String>,
    ) {
        if cancel
            .map(|c| c.load(std::sync::atomic::Ordering::SeqCst))
            .unwrap_or(false)
        {
            return;
        }
        // Load allowed extensions from systeminfo.txt in this dir if present.
        let mut dir_allowed = allowed_exts.clone();
        if let Ok(mut f) = sftp
            .open(&format!("{}/systeminfo.txt", dir.trim_end_matches('/')))
            .await
        {
            use tokio::io::AsyncReadExt;
            let mut buf = Vec::new();
            let _ = f.read_to_end(&mut buf).await;
            if let Ok(content) = String::from_utf8(buf)
                && let Some(exts) = crate::identify::parse_systeminfo_txt(&content)
                && !exts.is_empty()
            {
                dir_allowed = exts;
            }
        }

        if let Ok(entries) = sftp.read_dir(dir).await {
            for e in entries {
                if cancel
                    .map(|c| c.load(std::sync::atomic::Ordering::SeqCst))
                    .unwrap_or(false)
                {
                    return;
                }
                let name = e.file_name();
                if name == "." || name == ".." {
                    continue;
                }
                let meta = e.metadata();
                let is_dir = meta.is_dir();
                let full = if dir.ends_with('/') {
                    format!("{}{}", dir, name)
                } else {
                    format!("{}/{}", dir, name)
                };
                // Skip media/asset directories (whole segments only, not /run/media mount)
                let segs: Vec<&str> = full.split('/').collect();
                let skip = segs.iter().enumerate().any(|(i, seg)| {
                    (*seg == "media" || *seg == "images" || *seg == "assets")
                        && i > 0
                        && segs[i - 1] != "run"
                });
                if skip {
                    continue;
                }
                if is_dir {
                    Box::pin(walk(
                        sftp,
                        &full,
                        platforms_ext,
                        platform_slugs,
                        dir_allowed.clone(),
                        out,
                        app,
                        source_id,
                        cancel,
                        platform_filter,
                    ))
                    .await;
                } else {
                    let ext = std::path::Path::new(&name)
                        .extension()
                        .and_then(|x| x.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    let ext_dot = format!(".{}", ext);
                    let ext_known = platforms_ext
                        .iter()
                        .any(|x| x.eq_ignore_ascii_case(&ext_dot));
                    // Platform from folder name (e.g. .../roms/nes/...)
                    let folder_platform = full.split('/').any(|seg| {
                        let low = seg.to_lowercase();
                        platform_slugs.iter().any(|s| s == &low)
                    });
                    let ext_ok = if !dir_allowed.is_empty() {
                        dir_allowed.iter().any(|a| a.eq_ignore_ascii_case(&ext_dot))
                    } else {
                        ext_known || folder_platform
                    };
                    // Platform filter
                    let pf_ok = match platform_filter {
                        Some(pf) => full
                            .to_lowercase()
                            .contains(&format!("/{}/", pf.to_lowercase())),
                        None => true,
                    };
                    if ext_ok && pf_ok {
                        let size = meta.size.unwrap_or(0) as i64;
                        let mtime = meta.mtime.unwrap_or(0) as i64;
                        out.push((full, size, mtime, dir_allowed.clone()));
                        if let Some(a) = app
                            && out.len().is_multiple_of(500)
                        {
                            use tauri::Emitter;
                            let _ = a.emit("scan-progress", serde_json::json!({"source_id": source_id, "scanned": out.len(), "total": 0, "status": "scanning", "folder": dir, "file": name}));
                        }
                    }
                }
            }
        }
    }

    // Load gamelist known games (best-effort) from saves/es-de/gamelists/.
    let mut gamelist_known: std::collections::HashSet<String> = std::collections::HashSet::new();
    let emulation_root = if base_path.ends_with("roms/") || base_path.ends_with("roms") {
        base_path
            .trim_end_matches('/')
            .trim_end_matches("roms")
            .trim_end_matches('/')
    } else {
        base_path.trim_end_matches('/')
    };
    if let Ok(rd) = sess
        .sftp
        .read_dir(&format!("{}/saves/es-de/gamelists", emulation_root))
        .await
    {
        let dirs: Vec<_> = rd.collect();
        for d in dirs {
            if !d.metadata().is_dir() {
                continue;
            }
            let platform = d.file_name();
            let gp = format!(
                "{}/saves/es-de/gamelists/{}/gamelist.xml",
                emulation_root, platform
            );
            if let Ok(mut f) = sess.sftp.open(&gp).await {
                use tokio::io::AsyncReadExt;
                let mut buf = Vec::new();
                let _ = f.read_to_end(&mut buf).await;
                if let Ok(content) = String::from_utf8(buf) {
                    gamelist_known.extend(crate::identify::parse_gamelist_paths(&content));
                }
            }
        }
    }

    // Try to load systems.txt from the (possibly roms/) folder.
    if let Ok(mut f) = sess
        .sftp
        .open(&format!("{}/systems.txt", base_path.trim_end_matches('/')))
        .await
    {
        use tokio::io::AsyncReadExt;
        let mut buf = Vec::new();
        let _ = f.read_to_end(&mut buf).await;
        if let Ok(content) = String::from_utf8(buf) {
            let systems = crate::identify::parse_systems_txt(&content);
            let conn = state.db.lock().unwrap();
            let _ = crate::identify::ensure_platforms(&conn, &systems);
        }
    }

    let platforms = {
        let conn = state.db.lock().unwrap();
        crate::identify::load_platforms(&conn)?
    };
    let exts: Vec<String> = platforms
        .iter()
        .flat_map(|p| p.extensions.clone())
        .collect();
    let slugs: Vec<String> = platforms.iter().map(|p| p.slug.clone()).collect();
    walk(
        &sess.sftp,
        &base_path,
        &exts,
        &slugs,
        Vec::new(),
        &mut collected,
        app.as_ref(),
        &source_id,
        Some(&cancel),
        platform_filter.as_ref(),
    )
    .await;

    // process into db in chunks to avoid locking the DB for too long
    let mut files_found = 0usize;
    let mut games_created = 0usize;
    let mut games_updated = 0usize;
    let chunk_size = 500usize;
    let platforms2 = {
        let conn = state.db.lock().unwrap();
        crate::identify::load_platforms(&conn)?
    };

    for chunk in collected.chunks(chunk_size) {
        if cancel.load(std::sync::atomic::Ordering::SeqCst) {
            // cancelled: emit and return partial
            if let Some(a) = &app {
                use tauri::Emitter;
                let _ = a.emit("scan-progress", serde_json::json!({"source_id": source_id, "scanned": files_found, "total": collected.len(), "status": "cancelled"}));
            }
            return Ok(ScanResult {
                source_id,
                files_found,
                games_created,
                games_updated,
            });
        }
        // Emit progress for this chunk.
        if let Some(a) = &app {
            use tauri::Emitter;
            let _ = a.emit("scan-progress", serde_json::json!({"source_id": source_id, "scanned": files_found, "total": collected.len(), "status": "scanning"}));
        }

        let mut conn = state.db.lock().unwrap();
        let tx = conn.transaction()?;
        let mut last_file = String::new();
        for (fpath, size, mtime, _allowed) in chunk {
            let filename = std::path::Path::new(fpath)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            last_file = filename.clone();
            let ext = crate::identify::ext_of(fpath);
            let platform_id = if let Some(pf) = &platform_filter {
                Some(pf.clone())
            } else {
                crate::identify::infer_platform_from_path(fpath, &platforms2)
                    .or_else(|| crate::identify::guess_platform(&ext, &platforms2))
            };

            let existing: Option<(String, i64, i64)> = tx
                .query_row(
                    "SELECT id,size,modified_at FROM game_files WHERE source_id=?1 AND path=?2",
                    params![source_id, fpath],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .ok();

            // Incremental: skip unchanged files only if hashes already present.
            let needs_hash = existing.as_ref().map(|_| {
                // check hashes
                tx.query_row(
                    "SELECT (hash IS NULL OR crc32 IS NULL OR md5 IS NULL OR sha1 IS NULL) FROM game_files WHERE source_id=?1 AND path=?2",
                    params![source_id, fpath],
                    |r| r.get::<_, i64>(0).map(|v| v != 0),
                )
                .unwrap_or(true)
            }).unwrap_or(true);
            // Also reprocess if platform_id is wrong.
            let platform_ok = if let Some((_id, _, _)) = &existing {
                let stored: Option<String> = tx
                    .query_row(
                        "SELECT platform_id FROM game_files WHERE source_id=?1 AND path=?2",
                        params![source_id, fpath],
                        |r| r.get(0),
                    )
                    .ok()
                    .flatten();
                stored == platform_id
            } else {
                true
            };
            if let Some((_id, old_size, old_mtime)) = &existing
                && *old_size == *size
                && *old_mtime == *mtime
                && !needs_hash
                && platform_ok
            {
                files_found += 1;
                continue;
            }

            let file_id = if let Some((id, _, _)) = existing {
                tx.execute(
                    "UPDATE game_files SET filename=?1,extension=?2,size=?3,modified_at=?4,platform_id=?5,missing=0 WHERE id=?6",
                    params![filename, ext, *size, *mtime, platform_id, id],
                )?;
                id
            } else {
                let id = crate::models::new_id();
                tx.execute(
                    "INSERT INTO game_files (id,source_id,path,filename,extension,size,modified_at,platform_id,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                    params![id, source_id, fpath, filename, ext, *size, *mtime, platform_id, Utc::now().to_rfc3339()],
                )?;
                id
            };
            files_found += 1;

            // Skip game creation if already in ES-DE gamelist.
            if gamelist_known.contains(&filename.to_lowercase()) {
                continue;
            }

            let gf = crate::models::GameFile {
                id: file_id.clone(),
                source_id: source_id.clone(),
                path: fpath.clone(),
                filename: filename.clone(),
                extension: ext.clone(),
                size: *size,
                modified_at: *mtime,
                hash: None,
                crc32: None,
                md5: None,
                sha1: None,
                zip_inner: None,
                archive_hash: None,
                archive_crc32: None,
                archive_md5: None,
                archive_sha1: None,
                platform_id: platform_id.clone(),
                created_at: Utc::now(),
            };
            let (title, conf) = crate::identify::identify_file(&gf, &platforms2);

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
        }
        tx.commit()?;
        if let Some(a) = &app {
            use tauri::Emitter;
            let _ = a.emit("scan-progress", serde_json::json!({"source_id": source_id, "scanned": files_found, "total": collected.len(), "status": "scanning", "file": last_file}));
        }
    }

    // Mark missing files in one transaction.
    {
        let mut conn = state.db.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute(
            "UPDATE game_files SET missing=1 WHERE source_id=?1",
            params![source_id],
        )?;
        for chunk in collected.chunks(900) {
            // build placeholders
            let mut sql =
                String::from("UPDATE game_files SET missing=0 WHERE source_id=?1 AND path IN (");
            let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();
            params_vec.push(rusqlite::types::Value::Text(source_id.clone()));
            for (i, (fp, _, _, _)) in chunk.iter().enumerate() {
                if i > 0 {
                    sql.push(',');
                }
                sql.push('?');
                params_vec.push(rusqlite::types::Value::Text(fp.clone()));
            }
            sql.push(')');
            tx.execute(&sql, rusqlite::params_from_iter(params_vec))?;
        }
        tx.execute(
            "UPDATE game_sources SET last_scan_at=?1, available=1, last_error=NULL, root_mtime=?2 WHERE id=?3",
            params![Utc::now().to_rfc3339(), current_mtime, source_id],
        )?;
        let job_id = crate::models::new_id();
        tx.execute(
            "INSERT INTO sync_jobs (id,source_id,status,started_at,finished_at,files_found) VALUES (?1,?2,?3,?4,?5,?6)",
            params![job_id, source_id, "completed", Utc::now().to_rfc3339(), Utc::now().to_rfc3339(), files_found as i64],
        )?;
        tx.commit()?;
    }

    // Clean up orphan games.
    {
        let conn = state.db.lock().unwrap();
        let _ = crate::db::cleanup_orphan_games(&conn);
    }

    if let Some(a) = &app {
        use tauri::Emitter;
        let _ = a.emit("scan-progress", serde_json::json!({"source_id": source_id, "scanned": files_found, "total": files_found, "status": "done", "files_found": files_found, "games_created": games_created, "games_updated": games_updated}));
    }

    let _ = sess
        .session
        .disconnect(russh::Disconnect::ByApplication, "", "")
        .await;

    tracing::info!(
        "scan_remote_source done source_id={} files={} created={} updated={}",
        source_id,
        files_found,
        games_created,
        games_updated
    );
    Ok(ScanResult {
        source_id,
        files_found,
        games_created,
        games_updated,
    })
}

#[tauri::command]
pub async fn scan_remote_source(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    source_id: String,
    folder: Option<String>,
    platform_filter: Option<String>,
) -> Result<ScanResult> {
    state
        .cancel_scan
        .store(false, std::sync::atomic::Ordering::SeqCst);
    scan_remote_source_inner(
        &state,
        source_id,
        Some(app),
        state.cancel_scan.clone(),
        folder,
        platform_filter,
    )
    .await
}

#[tauri::command]
pub async fn scan_remote_folder(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    source_id: String,
    folder_path: String,
    platform_filter: Option<String>,
) -> Result<ScanResult> {
    state
        .cancel_scan
        .store(false, std::sync::atomic::Ordering::SeqCst);
    scan_remote_source_inner(
        &state,
        source_id,
        Some(app),
        state.cancel_scan.clone(),
        Some(folder_path),
        platform_filter,
    )
    .await
}

#[tauri::command]
pub async fn copy_remote_files(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    source_id: String,
    file_ids: Vec<String>,
    dest_dir: String,
) -> Result<TransferResult> {
    use tauri::Emitter;
    // gather connection + file info
    let (host, port, username, key_path) = {
        let conn = state.db.lock().unwrap();
        let cid: Option<String> = conn.query_row(
            "SELECT connection_id FROM game_sources WHERE id=?1",
            [source_id.clone()],
            |r| r.get(0),
        )?;
        let cid = cid.ok_or_else(|| RetroError::Invalid("not remote".into()))?;
        conn.query_row(
            "SELECT host,port,username,key_path FROM remote_connections WHERE id=?1",
            [cid],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)? as u16,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            },
        )?
    };

    let mut files_info: Vec<(String, String, i64)> = Vec::new(); // file_id, remote_path, size
    {
        let conn = state.db.lock().unwrap();
        for fid in &file_ids {
            let (path, size): (String, i64) = conn.query_row(
                "SELECT path,size FROM game_files WHERE id=?1",
                [fid.clone()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            files_info.push((fid.clone(), path, size));
        }
    }

    std::fs::create_dir_all(&dest_dir).map_err(RetroError::Io)?;

    let sess = remote::connect(&host, port, &username, &key_path).await?;

    let job_id = crate::models::new_id();
    {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO transfer_jobs (id,status,created_at,total_bytes) VALUES (?1,?2,?3,?4)",
            params![
                job_id,
                "running",
                Utc::now().to_rfc3339(),
                files_info.iter().map(|f| f.2).sum::<i64>()
            ],
        )?;
    }

    let mut errors = Vec::new();
    let mut total_bytes = 0i64;
    let mut done_files = 0usize;

    for (_fid, rpath, size) in &files_info {
        let filename = std::path::Path::new(rpath)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let dest = std::path::Path::new(&dest_dir).join(&filename);
        // skip if exists with same size
        if dest.exists()
            && let Ok(meta) = std::fs::metadata(&dest)
            && meta.len() as i64 == *size
        {
            total_bytes += size;
            done_files += 1;
            let _ = app.emit(
                "transfer-progress",
                serde_json::json!({"job_id": job_id, "file": filename, "status": "skipped"}),
            );
            continue;
        }

        // download
        match sess.sftp.open(rpath).await {
            Ok(mut rf) => {
                use tokio::io::AsyncReadExt;
                let mut buf = Vec::with_capacity(*size as usize);
                let mut tmp = vec![0u8; 65536];
                loop {
                    match rf.read(&mut tmp).await {
                        Ok(0) => break,
                        Ok(n) => buf.extend_from_slice(&tmp[..n]),
                        Err(e) => {
                            errors.push(format!("{}: {}", filename, e));
                            break;
                        }
                    }
                }
                // only write if no read error for this file
                let read_ok = !errors
                    .iter()
                    .any(|e| e.starts_with(&format!("{}:", filename)));
                if read_ok {
                    if let Err(e) = std::fs::write(&dest, &buf) {
                        errors.push(format!("{}: {}", filename, e));
                    } else {
                        total_bytes += buf.len() as i64;
                        done_files += 1;
                        let _ = app.emit("transfer-progress", serde_json::json!({"job_id": job_id, "file": filename, "status": "done", "bytes": buf.len()}));
                    }
                }
            }
            Err(e) => errors.push(format!("{}: {}", filename, e)),
        }
    }

    let status = if errors.is_empty() {
        "completed"
    } else {
        "completed_with_errors"
    };
    {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "UPDATE transfer_jobs SET status=?1, transferred_bytes=?2, error=?3 WHERE id=?4",
            params![
                status,
                total_bytes,
                if errors.is_empty() {
                    None
                } else {
                    Some(errors.join("; "))
                },
                job_id
            ],
        )?;
    }

    let _ = sess
        .session
        .disconnect(russh::Disconnect::ByApplication, "", "")
        .await;

    Ok(TransferResult {
        job_id,
        files: done_files,
        bytes: total_bytes,
        errors,
    })
}
