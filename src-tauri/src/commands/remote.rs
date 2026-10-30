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
    // validate key path
    if !std::path::Path::new(&key_path).exists() {
        return Err(RetroError::Invalid("key file not found".into()));
    }
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
    if !std::path::Path::new(&key_path).exists() {
        return Err(RetroError::Invalid("key file not found".into()));
    }
    // test connection first
    let sess = remote::connect(&host, port, &username, &key_path).await?;
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

#[tauri::command]
pub async fn scan_remote_source(
    state: State<'_, AppState>,
    source_id: String,
) -> Result<ScanResult> {
    // gather connection info
    let (host, port, username, key_path, base_path) = {
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

    let sess = remote::connect(&host, port, &username, &key_path).await?;

    // recursive collect files via sftp
    let mut collected: Vec<(String, i64, i64)> = Vec::new(); // path, size, mtime
    async fn walk(
        sftp: &russh_sftp::client::SftpSession,
        dir: &str,
        platforms_ext: &[String],
        out: &mut Vec<(String, i64, i64)>,
    ) {
        if let Ok(entries) = sftp.read_dir(dir).await {
            for e in entries {
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
                if is_dir {
                    Box::pin(walk(sftp, &full, platforms_ext, out)).await;
                } else {
                    let ext = std::path::Path::new(&name)
                        .extension()
                        .and_then(|x| x.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    let ext_dot = format!(".{}", ext);
                    if platforms_ext
                        .iter()
                        .any(|x| x.eq_ignore_ascii_case(&ext_dot))
                    {
                        let size = meta.size.unwrap_or(0) as i64;
                        let mtime = meta.mtime.unwrap_or(0) as i64;
                        out.push((full, size, mtime));
                    }
                }
            }
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

    walk(&sess.sftp, &base_path, &exts, &mut collected).await;

    // process into db (sync), like scanner but for remote source
    let mut files_found = 0usize;
    let mut games_created = 0usize;
    let mut games_updated = 0usize;
    {
        let mut conn = state.db.lock().unwrap();
        let platforms2 = crate::identify::load_platforms(&conn)?;

        let tx = conn.transaction()?;
        for (fpath, size, mtime) in &collected {
            let filename = std::path::Path::new(fpath)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let ext = crate::identify::ext_of(fpath);
            let platform_id = crate::identify::guess_platform(&ext, &platforms2);

            let existing: Option<String> = tx
                .query_row(
                    "SELECT id FROM game_files WHERE source_id=?1 AND path=?2",
                    params![source_id, fpath],
                    |r| r.get(0),
                )
                .ok();

            let file_id = if let Some(id) = existing {
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

            let gf = crate::models::GameFile {
                id: file_id.clone(),
                source_id: source_id.clone(),
                path: fpath.clone(),
                filename: filename.clone(),
                extension: ext.clone(),
                size: *size,
                modified_at: *mtime,
                hash: None,
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

        // mark missing files
        tx.execute(
            "UPDATE game_files SET missing=1 WHERE source_id=?1",
            params![source_id],
        )?;
        for (fpath, _, _) in &collected {
            tx.execute(
                "UPDATE game_files SET missing=0 WHERE source_id=?1 AND path=?2",
                params![source_id, fpath],
            )?;
        }

        tx.execute(
            "UPDATE game_sources SET last_scan_at=?1, available=1, last_error=NULL WHERE id=?2",
            params![Utc::now().to_rfc3339(), source_id],
        )?;

        let job_id = crate::models::new_id();
        tx.execute(
        "INSERT INTO sync_jobs (id,source_id,status,started_at,finished_at,files_found) VALUES (?1,?2,?3,?4,?5,?6)",
        params![job_id, source_id, "completed", Utc::now().to_rfc3339(), Utc::now().to_rfc3339(), files_found as i64],
    )?;

        tx.commit()?;
    }

    let _ = sess
        .session
        .disconnect(russh::Disconnect::ByApplication, "", "")
        .await;

    Ok(ScanResult {
        source_id,
        files_found,
        games_created,
        games_updated,
    })
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
