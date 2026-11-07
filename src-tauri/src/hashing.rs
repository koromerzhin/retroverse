use crate::error::{Result, RetroError};
use crate::state::AppState;
use rusqlite::params;
use serde_json::json;

use tauri::State;

#[allow(clippy::type_complexity)]
pub async fn compute_hashes_for_game_inner(
    state: &AppState,
    app: Option<tauri::AppHandle>,
    game_id: String,
) -> Result<usize> {
    use tauri::Emitter;
    // gather files + source info
    type FileInfo = (String, String, String, Option<String>, Option<String>);
    let files: Vec<FileInfo> = {
        let conn = state.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT gf.id, gf.path, gf.source_id, gs.kind, gs.connection_id FROM game_files gf JOIN game_file_links l ON l.file_id = gf.id JOIN game_sources gs ON gs.id = gf.source_id WHERE l.game_id = ?1 AND (gf.hash IS NULL OR gf.crc32 IS NULL OR gf.md5 IS NULL OR gf.sha1 IS NULL OR gf.sha512 IS NULL OR ((gf.extension='zip' OR gf.extension='7z') AND gf.archive_hash IS NULL) OR gf.hashed_mtime IS NULL OR gf.hashed_mtime != gf.modified_at OR gf.hashed_size != gf.size)",
        )?;
        let rows = stmt.query_map([game_id.clone()], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };

    // Fetch game title and platform for progress reporting.
    let (game_title, platform_name): (String, Option<String>) = {
        let conn = state.db.lock().unwrap();
        conn.query_row(
            "SELECT g.title, p.name FROM games g LEFT JOIN platforms p ON p.id = g.platform_id WHERE g.id = ?1",
            [game_id.clone()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).unwrap_or_else(|_| (game_id.clone(), None))
    };

    let total = files.len();
    if let Some(a) = &app {
        let _ = a.emit("hash-progress", serde_json::json!({"scanned": 0, "total": total, "status": "scanning", "game": game_title, "platform": platform_name, "file": null}));
    }

    let mut done = 0usize;
    for (fid, path, _source_id, kind, connection_id) in files {
        if state.cancel_hash.load(std::sync::atomic::Ordering::SeqCst) {
            if let Some(a) = &app {
                let _ = a.emit("hash-progress", serde_json::json!({"scanned": done, "total": total, "status": "cancelled", "game": game_title, "platform": platform_name, "file": path}));
            }
            return Ok(done);
        }
        let file_size: i64 = {
            let conn = state.db.lock().unwrap();
            conn.query_row(
                "SELECT size FROM game_files WHERE id=?1",
                [fid.clone()],
                |r| r.get(0),
            )
            .unwrap_or(0)
        };
        let started_at = chrono::Utc::now().timestamp_millis();
        if let Some(a) = &app {
            let _ = a.emit("hash-progress", serde_json::json!({"scanned": done, "total": total, "status": "reading", "game": game_title, "platform": platform_name, "file": path, "size": file_size, "started_at": started_at}));
        }
        let allowed: Vec<String> = Vec::new();
        let mut arch_hashes: (
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ) = (None, None, None, None, None);
        let result = if kind.as_deref() == Some("remote") {
            // download file via SFTP
            let (host, port, username, key_path) = {
                let conn = state.db.lock().unwrap();
                let cid = connection_id.ok_or_else(|| RetroError::Invalid("not remote".into()))?;
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
            let sess = crate::remote::connect(&host, port, &username, &key_path).await?;
            let remote_path = crate::identify::remote_path_from_url(&path);
            // Stream download to a temp file to avoid loading the whole file into memory.
            let tmp = std::env::temp_dir().join(format!("retroverse_{}", fid));
            {
                use tokio::io::AsyncReadExt;
                let mut f = sess
                    .sftp
                    .open(remote_path)
                    .await
                    .map_err(|e| RetroError::Other(e.to_string()))?;
                let mut out = std::fs::File::create(&tmp).map_err(RetroError::Io)?;
                use std::io::Write;
                let mut chunk = vec![0u8; 65536];
                loop {
                    let n = f
                        .read(&mut chunk)
                        .await
                        .map_err(|e| RetroError::Other(e.to_string()))?;
                    if n == 0 {
                        break;
                    }
                    out.write_all(&chunk[..n]).map_err(RetroError::Io)?;
                }
                out.flush().map_err(RetroError::Io)?;
            }
            let _ = sess
                .session
                .disconnect(russh::Disconnect::ByApplication, "", "")
                .await;
            let r = crate::identify::compute_hashes(tmp.to_str().unwrap_or(""), &allowed);
            let lower = remote_path.to_lowercase();
            let is_archive = lower.ends_with(".7z") || lower.ends_with(".zip");
            if is_archive {
                arch_hashes = crate::identify::hash_file_all(tmp.to_str().unwrap_or(""));
            }
            let _ = std::fs::remove_file(&tmp);
            r
        } else {
            crate::identify::compute_hashes(&path, &allowed)
        };

        let (sha256, crc32, md5, sha1, zip_inner, sha512) = result;
        let lower = path.to_lowercase();
        let is_archive = lower.ends_with(".7z") || lower.ends_with(".zip");
        let (arch_sha, arch_crc, arch_md5, arch_sha1, arch_sha512) = if is_archive {
            if kind.as_deref() == Some("remote") {
                arch_hashes
            } else {
                crate::identify::hash_file_all(&path)
            }
        } else {
            (None, None, None, None, None)
        };
        if sha256.is_some() {
            let conn = state.db.lock().unwrap();
            conn.execute(
                "UPDATE game_files SET hash=?1,crc32=?2,md5=?3,sha1=?4,zip_inner=?5,sha512=?6,archive_hash=?7,archive_crc32=?8,archive_md5=?9,archive_sha1=?10,archive_sha512=?11,hashed_mtime=modified_at,hashed_size=size WHERE id=?12",
                params![sha256, crc32, md5, sha1, zip_inner, sha512, arch_sha, arch_crc, arch_md5, arch_sha1, arch_sha512, fid],
            )?;
            done += 1;
            if let Some(a) = &app {
                let _ = a.emit("hash-progress", serde_json::json!({"scanned": done, "total": total, "status": "file_done", "game": game_title, "platform": platform_name, "file": path, "size": file_size, "started_at": started_at, "ok": true}));
            }
        }
    }
    if let Some(a) = &app {
        let _ = a.emit("hash-progress", serde_json::json!({"scanned": total, "total": total, "status": "done", "done": done, "game": game_title, "platform": platform_name, "file": null}));
    }
    Ok(done)
}

pub async fn compute_hashes_for_game(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    game_id: String,
) -> Result<usize> {
    compute_hashes_for_game_inner(&state, Some(app), game_id).await
}

pub fn list_transfer_jobs(state: State<AppState>) -> Result<Vec<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id,status,created_at,total_bytes,transferred_bytes,error FROM transfer_jobs ORDER BY created_at DESC")?;
    let rows = stmt.query_map([], |r| {
        Ok(json!({
            "id": r.get::<_,String>(0)?,
            "status": r.get::<_,String>(1)?,
            "created_at": r.get::<_,String>(2)?,
            "total_bytes": r.get::<_,i64>(3)?,
            "transferred_bytes": r.get::<_,i64>(4)?,
            "error": r.get::<_,Option<String>>(5)?,
        }))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub async fn compute_hashes_for_file(state: State<'_, AppState>, file_id: String) -> Result<usize> {
    compute_hash_for_file_inner(&state, file_id).await
}

pub async fn compute_hash_for_file_inner(state: &AppState, file_id: String) -> Result<usize> {
    use russh::Disconnect;

    let (path, kind, connection_id): (String, Option<String>, Option<String>) = {
        let conn = state.db.lock().unwrap();
        conn.query_row(
            "SELECT gf.path, gs.kind, gs.connection_id FROM game_files gf JOIN game_sources gs ON gs.id = gf.source_id WHERE gf.id = ?1",
            [file_id.clone()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?
    };

    let kind = kind.unwrap_or_else(|| "local".to_string());
    let allowed: Vec<String> = Vec::new();

    let (
        sha256,
        crc32,
        md5,
        sha1,
        zip_inner,
        sha512,
        arch_sha,
        arch_crc,
        arch_md5,
        arch_sha1,
        arch_sha512,
    ) = if kind == "remote" {
        let (host, port, username, key_path) = {
            let conn = state.db.lock().unwrap();
            let cid = connection_id.ok_or_else(|| RetroError::Invalid("not remote".into()))?;
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
        let sess = crate::remote::connect(&host, port, &username, &key_path).await?;
        let remote_path = crate::identify::remote_path_from_url(&path);
        let tmp = std::env::temp_dir().join(format!("retroverse_{}", file_id));
        {
            use tokio::io::AsyncReadExt;
            let mut f = sess
                .sftp
                .open(remote_path)
                .await
                .map_err(|e| RetroError::Other(e.to_string()))?;
            let mut out = std::fs::File::create(&tmp).map_err(RetroError::Io)?;
            use std::io::Write;
            let mut chunk = vec![0u8; 65536];
            loop {
                let n = f
                    .read(&mut chunk)
                    .await
                    .map_err(|e| RetroError::Other(e.to_string()))?;
                if n == 0 {
                    break;
                }
                out.write_all(&chunk[..n]).map_err(RetroError::Io)?;
            }
            out.flush().map_err(RetroError::Io)?;
        }
        let _ = sess
            .session
            .disconnect(Disconnect::ByApplication, "", "")
            .await;
        let content = crate::identify::compute_hashes(tmp.to_str().unwrap_or(""), &allowed);
        let (s, c, m, h, zi, sh512) = content;
        let lower = crate::identify::remote_path_from_url(&path).to_lowercase();
        let is_archive = lower.ends_with(".7z") || lower.ends_with(".zip");
        let (as_, ac, am, ah, ash512) = if is_archive {
            crate::identify::hash_file_all(tmp.to_str().unwrap_or(""))
        } else {
            (None, None, None, None, None)
        };
        let _ = std::fs::remove_file(&tmp);
        (s, c, m, h, zi, sh512, as_, ac, am, ah, ash512)
    } else {
        let content = crate::identify::compute_hashes(&path, &allowed);
        let (s, c, m, h, zi, sh512) = content;
        let lower = crate::identify::remote_path_from_url(&path).to_lowercase();
        let is_archive = lower.ends_with(".7z") || lower.ends_with(".zip");
        let (as_, ac, am, ah, ash512) = if is_archive {
            crate::identify::hash_file_all(&path)
        } else {
            (None, None, None, None, None)
        };
        (s, c, m, h, zi, sh512, as_, ac, am, ah, ash512)
    };

    if sha256.is_some() {
        let conn = state.db.lock().unwrap();
        let _ = conn.execute(
            "UPDATE game_files SET hash=?1,crc32=?2,md5=?3,sha1=?4,zip_inner=?5,sha512=?6,archive_hash=?7,archive_crc32=?8,archive_md5=?9,archive_sha1=?10,archive_sha512=?11,hashed_mtime=modified_at,hashed_size=size WHERE id=?12",
            rusqlite::params![sha256, crc32, md5, sha1, zip_inner, sha512, arch_sha, arch_crc, arch_md5, arch_sha1, arch_sha512, file_id],
        );
        Ok(1)
    } else {
        Ok(0)
    }
}

pub async fn compute_hashes_for_platform(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    platform_id: String,
) -> Result<usize> {
    use tauri::Emitter;
    state
        .cancel_hash
        .store(false, std::sync::atomic::Ordering::SeqCst);
    let cancel = state.cancel_hash.clone();

    let ids: Vec<String> = {
        let conn = state.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id FROM game_files WHERE platform_id=?1 AND (hash IS NULL OR crc32 IS NULL OR md5 IS NULL OR sha1 IS NULL OR sha512 IS NULL OR ((extension='zip' OR extension='7z') AND archive_hash IS NULL) OR hashed_mtime IS NULL OR hashed_mtime != modified_at OR hashed_size != size)",
        )?;
        stmt.query_map([platform_id.clone()], |r| r.get(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?
    };

    let total = ids.len();
    let mut done = 0usize;
    let _ = app.emit(
        "hash-progress",
        serde_json::json!({"scanned": 0, "total": total, "status": "scanning"}),
    );

    for id in ids.into_iter() {
        if cancel.load(std::sync::atomic::Ordering::SeqCst) {
            let _ = app.emit(
                "hash-progress",
                serde_json::json!({"scanned": done, "total": total, "status": "cancelled"}),
            );
            return Ok(done);
        }
        // Fetch file info for progress reporting.
        let (path, game_title, platform_name, file_size): (String, String, Option<String>, i64) = {
            let conn = state.db.lock().unwrap();
            conn.query_row(
                "SELECT gf.path, COALESCE(g.title, '—'), p.name, gf.size FROM game_files gf LEFT JOIN game_file_links l ON l.file_id = gf.id LEFT JOIN games g ON g.id = l.game_id LEFT JOIN platforms p ON p.id = g.platform_id WHERE gf.id = ?1",
                [id.clone()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get::<_, i64>(3).unwrap_or(0))),
            ).unwrap_or_else(|_| (id.clone(), "—".to_string(), None, 0))
        };
        let started_at = chrono::Utc::now().timestamp_millis();
        let _ = app.emit("hash-progress", serde_json::json!({"scanned": done, "total": total, "status": "reading", "game": game_title, "platform": platform_name, "file": path, "size": file_size, "started_at": started_at}));
        let n = compute_hash_for_file_inner(&state, id).await?;
        done += n;
        let _ = app.emit("hash-progress", serde_json::json!({"scanned": done, "total": total, "status": "file_done", "game": game_title, "platform": platform_name, "file": path, "size": file_size, "started_at": started_at, "ok": n > 0}));
    }
    let _ = app.emit(
        "hash-progress",
        serde_json::json!({"scanned": total, "total": total, "status": "done", "done": done}),
    );
    Ok(done)
}

pub fn unhashed_counts_by_platform(
    state: State<'_, AppState>,
) -> Result<std::collections::HashMap<String, i64>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT platform_id, COUNT(*) FROM game_files WHERE platform_id IS NOT NULL AND (hash IS NULL OR crc32 IS NULL OR md5 IS NULL OR sha1 IS NULL OR sha512 IS NULL OR ((extension='zip' OR extension='7z') AND archive_hash IS NULL) OR hashed_mtime IS NULL OR hashed_mtime != modified_at OR hashed_size != size) GROUP BY platform_id",
    )?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
    let mut map = std::collections::HashMap::new();
    for r in rows {
        let (k, v) = r?;
        map.insert(k, v);
    }
    Ok(map)
}

pub async fn compute_hashes_for_all_games(state: State<'_, AppState>) -> Result<usize> {
    state
        .cancel_hash
        .store(false, std::sync::atomic::Ordering::SeqCst);
    let game_ids: Vec<String> = {
        let conn = state.db.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id FROM games")?;
        let rows = stmt.query_map([], |r| r.get(0))?;
        rows.filter_map(|r| r.ok()).collect()
    };
    let mut total = 0usize;
    for gid in game_ids {
        if let Ok(n) = compute_hashes_for_game_inner(&state, None, gid).await {
            total += n;
        }
    }
    Ok(total)
}

pub fn cancel_hash(state: State<AppState>) -> Result<()> {
    state
        .cancel_hash
        .store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

/// Trait for hash computation strategies.
pub trait HashComputer {
    fn compute(&self, path: &str, allowed: &[String]) -> crate::identify::Hashes;
}

/// Default local file hasher.
pub struct LocalHasher;
impl HashComputer for LocalHasher {
    fn compute(&self, path: &str, allowed: &[String]) -> crate::identify::Hashes {
        crate::identify::compute_hashes(path, allowed)
    }
}
