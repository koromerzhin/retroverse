use crate::error::{Result, RetroError};
use crate::state::AppState;

type ArchiveHashes = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);
use rusqlite::params;
use serde_json::json;

use std::fs;
use tauri::State;

pub async fn compute_hashes_inner(
    state: &AppState,
    app: Option<tauri::AppHandle>,
    source_id: Option<String>,
) -> Result<usize> {
    use tauri::Emitter;
    state
        .cancel_hash
        .store(false, std::sync::atomic::Ordering::SeqCst);
    let cancel = state.cancel_hash.clone();

    let rows: Vec<(String, String)> = {
        let conn = state.db.lock().unwrap();
        if let Some(sid) = &source_id {
            let mut stmt = conn.prepare("SELECT id,path FROM game_files WHERE source_id=?1 AND (hash IS NULL OR crc32 IS NULL OR md5 IS NULL OR sha1 IS NULL OR sha512 IS NULL OR ((extension='zip' OR extension='7z') AND archive_hash IS NULL))")?;
            stmt.query_map([sid.clone()], |r| Ok((r.get(0)?, r.get(1)?)))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            let mut stmt = conn.prepare("SELECT id,path FROM game_files WHERE hash IS NULL OR crc32 IS NULL OR md5 IS NULL OR sha1 IS NULL OR sha512 IS NULL OR ((extension='zip' OR extension='7z') AND archive_hash IS NULL)")?;
            stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        }
    };

    // Map file_id -> (game_title, platform_name) for progress reporting.
    let file_game_map: std::collections::HashMap<String, (String, Option<String>, i64)> = {
        let conn = state.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT gf.id, g.title, p.name FROM game_files gf LEFT JOIN game_file_links l ON l.file_id = gf.id LEFT JOIN games g ON g.id = l.game_id LEFT JOIN platforms p ON p.id = g.platform_id",
        ).unwrap();
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, i64>(3).unwrap_or(0),
                ))
            })
            .unwrap();
        let mut m = std::collections::HashMap::new();
        for r in rows.filter_map(|r| r.ok()) {
            let (fid, title, plat, size) = r;
            m.insert(fid, (title.unwrap_or_else(|| "—".to_string()), plat, size));
        }
        m
    };

    let total = rows.len();
    let mut done = 0usize;
    if let Some(a) = &app {
        let _ = a.emit(
            "hash-progress",
            serde_json::json!({"scanned": 0, "total": total, "status": "scanning", "game": null, "platform": null, "file": null}),
        );
    }

    for (id, path) in rows.into_iter() {
        let (game_title, platform_name, file_size) = file_game_map
            .get(&id)
            .cloned()
            .unwrap_or_else(|| ("—".to_string(), None, 0));
        let started_at = chrono::Utc::now().timestamp_millis();
        if let Some(a) = &app {
            let _ = a.emit("hash-progress", serde_json::json!({"scanned": done, "total": total, "status": "reading", "game": game_title, "platform": platform_name, "file": path, "size": file_size, "started_at": started_at}));
        }
        if cancel.load(std::sync::atomic::Ordering::SeqCst) {
            if let Some(a) = &app {
                let _ = a.emit(
                    "hash-progress",
                    serde_json::json!({"scanned": done, "total": total, "status": "cancelled"}),
                );
            }
            return Ok(done);
        }
        let (sha256, crc32, md5, sha1, zip_inner, sha512) =
            crate::identify::compute_hashes(&path, &[]);
        let lower = path.to_lowercase();
        let is_archive = lower.ends_with(".zip") || lower.ends_with(".7z");
        let (arch_sha, arch_crc, arch_md5, arch_sha1, arch_sha512) = if is_archive {
            crate::identify::hash_file_all(&path)
        } else {
            (None, None, None, None, None)
        };
        if sha256.is_some() {
            let conn = state.db.lock().unwrap();
            let _ = conn.execute(
                "UPDATE game_files SET hash=?1,crc32=?2,md5=?3,sha1=?4,zip_inner=?5,sha512=?6,archive_hash=?7,archive_crc32=?8,archive_md5=?9,archive_sha1=?10,archive_sha512=?11 WHERE id=?12",
                params![sha256, crc32, md5, sha1, zip_inner, sha512, arch_sha, arch_crc, arch_md5, arch_sha1, arch_sha512, id],
            );
            done += 1;
        }
        if sha256.is_some() {
            done += 1;
        }
        if let Some(a) = &app {
            let _ = a.emit("hash-progress", serde_json::json!({"scanned": done, "total": total, "status": "file_done", "game": game_title, "platform": platform_name, "file": path, "size": file_size, "started_at": started_at, "ok": sha256.is_some()}));
        }
    }
    if let Some(a) = &app {
        let _ = a.emit(
            "hash-progress",
            serde_json::json!({"scanned": total, "total": total, "status": "done", "done": done}),
        );
    }
    Ok(done)
}

#[tauri::command]
pub async fn compute_hashes(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    source_id: Option<String>,
) -> Result<usize> {
    compute_hashes_inner(&state, Some(app), source_id).await
}

#[tauri::command]
pub fn export_backup(state: State<AppState>, dest_path: String) -> Result<String> {
    let conn = state.db.lock().unwrap();

    let sources: Vec<serde_json::Value> = {
        let mut stmt = conn.prepare("SELECT id,name,kind,path,created_at,last_scan_at,connection_id,remote_path,available,last_error FROM game_sources")?;
        let rows = stmt.query_map([], |r| {
            Ok(json!({
                "id": r.get::<_,String>(0)?,
                "name": r.get::<_,String>(1)?,
                "kind": r.get::<_,String>(2)?,
                "path": r.get::<_,String>(3)?,
                "created_at": r.get::<_,String>(4)?,
                "last_scan_at": r.get::<_,Option<String>>(5)?,
                "connection_id": r.get::<_,Option<String>>(6)?,
                "remote_path": r.get::<_,Option<String>>(7)?,
                "available": r.get::<_,i64>(8)? != 0,
                "last_error": r.get::<_,Option<String>>(9)?,
            }))
        })?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        v
    };

    let games: Vec<serde_json::Value> = {
        let mut stmt =
            conn.prepare("SELECT id,title,platform_id,created_at,updated_at FROM games")?;
        let rows = stmt.query_map([], |r| {
            Ok(json!({
                "id": r.get::<_,String>(0)?,
                "title": r.get::<_,String>(1)?,
                "platform_id": r.get::<_,Option<String>>(2)?,
                "created_at": r.get::<_,String>(3)?,
                "updated_at": r.get::<_,String>(4)?,
            }))
        })?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        v
    };

    let files: Vec<serde_json::Value> = {
        let mut stmt = conn.prepare("SELECT id,source_id,path,filename,extension,size,modified_at,hash,platform_id,created_at,missing FROM game_files")?;
        let rows = stmt.query_map([], |r| {
            Ok(json!({
                "id": r.get::<_,String>(0)?,
                "source_id": r.get::<_,String>(1)?,
                "path": r.get::<_,String>(2)?,
                "filename": r.get::<_,String>(3)?,
                "extension": r.get::<_,String>(4)?,
                "size": r.get::<_,i64>(5)?,
                "modified_at": r.get::<_,i64>(6)?,
                "hash": r.get::<_,Option<String>>(7)?,
                "platform_id": r.get::<_,Option<String>>(8)?,
                "created_at": r.get::<_,String>(9)?,
                "missing": r.get::<_,i64>(10)? != 0,
            }))
        })?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        v
    };

    let links: Vec<serde_json::Value> = {
        let mut stmt =
            conn.prepare("SELECT file_id,game_id,confidence,manual FROM game_file_links")?;
        let rows = stmt.query_map([], |r| {
            Ok(json!({
                "file_id": r.get::<_,String>(0)?,
                "game_id": r.get::<_,String>(1)?,
                "confidence": r.get::<_,f64>(2)?,
                "manual": r.get::<_,i64>(3)? != 0,
            }))
        })?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        v
    };

    let metadata: Vec<serde_json::Value> = {
        let mut stmt = conn.prepare(
            "SELECT id,game_id,provider,field,value,provenance,fetched_at FROM metadata_records",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(json!({
                "id": r.get::<_,String>(0)?,
                "game_id": r.get::<_,String>(1)?,
                "provider": r.get::<_,String>(2)?,
                "field": r.get::<_,String>(3)?,
                "value": r.get::<_,String>(4)?,
                "provenance": r.get::<_,Option<String>>(5)?,
                "fetched_at": r.get::<_,String>(6)?,
            }))
        })?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        v
    };

    let media: Vec<serde_json::Value> = {
        let mut stmt =
            conn.prepare("SELECT id,game_id,kind,path,url,cached_at FROM media_assets")?;
        let rows = stmt.query_map([], |r| {
            Ok(json!({
                "id": r.get::<_,String>(0)?,
                "game_id": r.get::<_,String>(1)?,
                "kind": r.get::<_,String>(2)?,
                "path": r.get::<_,Option<String>>(3)?,
                "url": r.get::<_,Option<String>>(4)?,
                "cached_at": r.get::<_,Option<String>>(5)?,
            }))
        })?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        v
    };

    let collections: Vec<serde_json::Value> = {
        let mut stmt = conn.prepare("SELECT id,name,created_at FROM collections")?;
        let rows = stmt.query_map([], |r| {
            Ok(json!({
                "id": r.get::<_,String>(0)?,
                "name": r.get::<_,String>(1)?,
                "created_at": r.get::<_,String>(2)?,
            }))
        })?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        v
    };

    let collection_games: Vec<serde_json::Value> = {
        let mut stmt = conn.prepare("SELECT collection_id,game_id FROM collection_games")?;
        let rows = stmt.query_map([], |r| {
            Ok(json!({
                "collection_id": r.get::<_,String>(0)?,
                "game_id": r.get::<_,String>(1)?,
            }))
        })?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        v
    };

    let backup = json!({
        "version": 1,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "sources": sources,
        "games": games,
        "files": files,
        "links": links,
        "metadata": metadata,
        "media": media,
        "collections": collections,
        "collection_games": collection_games,
    });

    let content = serde_json::to_string_pretty(&backup).map_err(RetroError::Serde)?;
    fs::write(&dest_path, content).map_err(RetroError::Io)?;
    Ok(dest_path)
}

#[tauri::command]
pub fn import_backup(state: State<AppState>, src_path: String) -> Result<usize> {
    let content = fs::read_to_string(&src_path).map_err(RetroError::Io)?;
    let v: serde_json::Value = serde_json::from_str(&content).map_err(RetroError::Serde)?;
    let conn = state.db.lock().unwrap();
    let mut imported = 0usize;

    if let Some(arr) = v["sources"].as_array() {
        for s in arr {
            let id = s["id"].as_str().unwrap_or("");
            if id.is_empty() {
                continue;
            }
            let exists: bool = conn
                .query_row("SELECT COUNT(*) FROM game_sources WHERE id=?1", [id], |r| {
                    r.get::<_, i64>(0)
                })
                .unwrap_or(0)
                > 0;
            if exists {
                continue;
            }
            conn.execute(
                "INSERT OR IGNORE INTO game_sources (id,name,kind,path,created_at,last_scan_at,connection_id,remote_path,available,last_error) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                params![
                    id,
                    s["name"].as_str().unwrap_or(""),
                    s["kind"].as_str().unwrap_or("local"),
                    s["path"].as_str().unwrap_or(""),
                    s["created_at"].as_str().unwrap_or(""),
                    s["last_scan_at"].as_str(),
                    s["connection_id"].as_str(),
                    s["remote_path"].as_str(),
                    s["available"].as_bool().unwrap_or(true) as i64,
                    s["last_error"].as_str(),
                ],
            )?;
            imported += 1;
        }
    }
    if let Some(arr) = v["games"].as_array() {
        for g in arr {
            let id = g["id"].as_str().unwrap_or("");
            if id.is_empty() {
                continue;
            }
            let exists: bool = conn
                .query_row("SELECT COUNT(*) FROM games WHERE id=?1", [id], |r| {
                    r.get::<_, i64>(0)
                })
                .unwrap_or(0)
                > 0;
            if exists {
                continue;
            }
            conn.execute(
                "INSERT OR IGNORE INTO games (id,title,platform_id,created_at,updated_at) VALUES (?1,?2,?3,?4,?5)",
                params![id, g["title"].as_str().unwrap_or(""), g["platform_id"].as_str(), g["created_at"].as_str().unwrap_or(""), g["updated_at"].as_str().unwrap_or("")],
            )?;
            imported += 1;
        }
    }
    if let Some(arr) = v["files"].as_array() {
        for f in arr {
            let id = f["id"].as_str().unwrap_or("");
            if id.is_empty() {
                continue;
            }
            let exists: bool = conn
                .query_row("SELECT COUNT(*) FROM game_files WHERE id=?1", [id], |r| {
                    r.get::<_, i64>(0)
                })
                .unwrap_or(0)
                > 0;
            if exists {
                continue;
            }
            conn.execute(
                "INSERT OR IGNORE INTO game_files (id,source_id,path,filename,extension,size,modified_at,hash,platform_id,created_at,missing) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![
                    id,
                    f["source_id"].as_str().unwrap_or(""),
                    f["path"].as_str().unwrap_or(""),
                    f["filename"].as_str().unwrap_or(""),
                    f["extension"].as_str().unwrap_or(""),
                    f["size"].as_i64().unwrap_or(0),
                    f["modified_at"].as_i64().unwrap_or(0),
                    f["hash"].as_str(),
                    f["platform_id"].as_str(),
                    f["created_at"].as_str().unwrap_or(""),
                    f["missing"].as_bool().unwrap_or(false) as i64,
                ],
            )?;
            imported += 1;
        }
    }
    if let Some(arr) = v["links"].as_array() {
        for l in arr {
            let fid = l["file_id"].as_str().unwrap_or("");
            let gid = l["game_id"].as_str().unwrap_or("");
            if fid.is_empty() || gid.is_empty() {
                continue;
            }
            conn.execute(
                "INSERT OR IGNORE INTO game_file_links (file_id,game_id,confidence,manual) VALUES (?1,?2,?3,?4)",
                params![fid, gid, l["confidence"].as_f64().unwrap_or(0.5), l["manual"].as_bool().unwrap_or(false) as i64],
            )?;
        }
    }
    if let Some(arr) = v["metadata"].as_array() {
        for m in arr {
            let id = m["id"].as_str().unwrap_or("");
            if id.is_empty() {
                continue;
            }
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM metadata_records WHERE id=?1",
                    [id],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap_or(0)
                > 0;
            if exists {
                continue;
            }
            conn.execute(
                "INSERT OR IGNORE INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                params![id, m["game_id"].as_str().unwrap_or(""), m["provider"].as_str().unwrap_or(""), m["field"].as_str().unwrap_or(""), m["value"].as_str().unwrap_or(""), m["provenance"].as_str(), m["fetched_at"].as_str().unwrap_or("")],
            )?;
        }
    }
    if let Some(arr) = v["media"].as_array() {
        for m in arr {
            let id = m["id"].as_str().unwrap_or("");
            if id.is_empty() {
                continue;
            }
            let exists: bool = conn
                .query_row("SELECT COUNT(*) FROM media_assets WHERE id=?1", [id], |r| {
                    r.get::<_, i64>(0)
                })
                .unwrap_or(0)
                > 0;
            if exists {
                continue;
            }
            conn.execute(
                "INSERT OR IGNORE INTO media_assets (id,game_id,kind,path,url,cached_at) VALUES (?1,?2,?3,?4,?5,?6)",
                params![id, m["game_id"].as_str().unwrap_or(""), m["kind"].as_str().unwrap_or(""), m["path"].as_str(), m["url"].as_str(), m["cached_at"].as_str()],
            )?;
        }
    }
    if let Some(arr) = v["collections"].as_array() {
        for c in arr {
            let id = c["id"].as_str().unwrap_or("");
            if id.is_empty() {
                continue;
            }
            conn.execute(
                "INSERT OR IGNORE INTO collections (id,name,created_at) VALUES (?1,?2,?3)",
                params![
                    id,
                    c["name"].as_str().unwrap_or(""),
                    c["created_at"].as_str().unwrap_or("")
                ],
            )?;
        }
    }
    if let Some(arr) = v["collection_games"].as_array() {
        for cg in arr {
            conn.execute(
                "INSERT OR IGNORE INTO collection_games (collection_id,game_id) VALUES (?1,?2)",
                params![
                    cg["collection_id"].as_str().unwrap_or(""),
                    cg["game_id"].as_str().unwrap_or("")
                ],
            )?;
        }
    }

    Ok(imported)
}

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
            "SELECT gf.id, gf.path, gf.source_id, gs.kind, gs.connection_id FROM game_files gf JOIN game_file_links l ON l.file_id = gf.id JOIN game_sources gs ON gs.id = gf.source_id WHERE l.game_id = ?1 AND (gf.hash IS NULL OR gf.crc32 IS NULL OR gf.md5 IS NULL OR gf.sha1 IS NULL OR gf.sha512 IS NULL OR ((gf.extension='zip' OR gf.extension='7z') AND gf.archive_hash IS NULL))",
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
        let mut arch_hashes: ArchiveHashes = (None, None, None, None, None);
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
            // Stream download to a temp file to avoid loading the whole file into memory.
            let tmp = std::env::temp_dir().join(format!("retroverse_{}", fid));
            {
                use tokio::io::AsyncReadExt;
                let mut f = sess
                    .sftp
                    .open(&path)
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
            let lower = path.to_lowercase();
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
                "UPDATE game_files SET hash=?1,crc32=?2,md5=?3,sha1=?4,zip_inner=?5,sha512=?6,archive_hash=?7,archive_crc32=?8,archive_md5=?9,archive_sha1=?10,archive_sha512=?11 WHERE id=?12",
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

#[tauri::command]
pub async fn compute_hashes_for_game(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    game_id: String,
) -> Result<usize> {
    compute_hashes_for_game_inner(&state, Some(app), game_id).await
}

#[tauri::command]
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

#[tauri::command]
pub async fn compute_hashes_for_file(state: State<'_, AppState>, file_id: String) -> Result<usize> {
    compute_hash_for_file_inner(&state, file_id).await
}

pub async fn compute_hash_for_file_inner(state: &AppState, file_id: String) -> Result<usize> {
    use crate::remote;
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
        let sess = remote::connect(&host, port, &username, &key_path).await?;
        let tmp = std::env::temp_dir().join(format!("retroverse_{}", file_id));
        {
            use tokio::io::AsyncReadExt;
            let mut f = sess
                .sftp
                .open(&path)
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
        let lower = path.to_lowercase();
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
        let lower = path.to_lowercase();
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
            "UPDATE game_files SET hash=?1,crc32=?2,md5=?3,sha1=?4,zip_inner=?5,sha512=?6,archive_hash=?7,archive_crc32=?8,archive_md5=?9,archive_sha1=?10,archive_sha512=?11 WHERE id=?12",
            rusqlite::params![sha256, crc32, md5, sha1, zip_inner, sha512, arch_sha, arch_crc, arch_md5, arch_sha1, arch_sha512, file_id],
        );
        Ok(1)
    } else {
        Ok(0)
    }
}

#[tauri::command]
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
            "SELECT id FROM game_files WHERE platform_id=?1 AND (hash IS NULL OR crc32 IS NULL OR md5 IS NULL OR sha1 IS NULL OR sha512 IS NULL OR ((extension='zip' OR extension='7z') AND archive_hash IS NULL))",
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

#[tauri::command]
pub fn unhashed_counts_by_platform(
    state: State<'_, AppState>,
) -> Result<std::collections::HashMap<String, i64>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT platform_id, COUNT(*) FROM game_files WHERE platform_id IS NOT NULL AND (hash IS NULL OR crc32 IS NULL OR md5 IS NULL OR sha1 IS NULL OR sha512 IS NULL OR ((extension='zip' OR extension='7z') AND archive_hash IS NULL)) GROUP BY platform_id",
    )?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
    let mut map = std::collections::HashMap::new();
    for r in rows {
        let (k, v) = r?;
        map.insert(k, v);
    }
    Ok(map)
}

#[tauri::command]
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

#[tauri::command]
pub fn cancel_hash(state: State<AppState>) -> Result<()> {
    state
        .cancel_hash
        .store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}
