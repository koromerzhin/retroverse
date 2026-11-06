use crate::error::{Result, RetroError};
use crate::models::{GameSource, ScanResult};
use crate::scanner;
use crate::state::AppState;
use chrono::Utc;
use rusqlite::params;
use std::sync::atomic::Ordering;
use tauri::State;

#[tauri::command]
pub fn list_sources(state: State<AppState>) -> Result<Vec<GameSource>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id,name,kind,path,created_at,last_scan_at,connection_id,remote_path,available,last_error,active FROM game_sources ORDER BY created_at DESC")?;
    let rows = stmt.query_map([], |r| {
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
            active: r.get::<_, i64>(10).unwrap_or(0) != 0,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[tauri::command]
pub fn add_source(state: State<AppState>, name: String, path: String) -> Result<GameSource> {
    // validate path
    let p = std::path::Path::new(&path);
    // Reject paths containing traversal components for safety.
    if p.components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(RetroError::Invalid(
            "path must not contain '..' components".into(),
        ));
    }
    if !p.exists() {
        return Err(RetroError::Invalid(format!(
            "path does not exist: {}",
            path
        )));
    }
    if !p.is_dir() {
        return Err(RetroError::Invalid("path must be a directory".into()));
    }
    // A source must point to a folder containing a 'roms' subfolder (or be 'roms' itself).
    let final_path = if p.ends_with("roms") {
        path.clone()
    } else {
        let roms = p.join("roms");
        if roms.is_dir() {
            roms.to_string_lossy().to_string()
        } else {
            return Err(RetroError::Invalid(
                "Le dossier doit contenir un sous-dossier 'roms'".into(),
            ));
        }
    };

    let id = crate::models::new_id();
    let now = Utc::now();
    {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO game_sources (id,name,kind,path,created_at) VALUES (?1,?2,?3,?4,?5)",
            params![id, name, "local", final_path, now.to_rfc3339()],
        )?;
    }
    Ok(GameSource {
        id,
        name,
        kind: "local".into(),
        path: final_path,
        created_at: now,
        last_scan_at: None,
        connection_id: None,
        remote_path: None,
        available: true,
        last_error: None,
        active: false,
    })
}

#[tauri::command]
pub fn remove_source(state: State<AppState>, id: String) -> Result<()> {
    let conn = state.db.lock().unwrap();
    conn.execute("DELETE FROM game_sources WHERE id = ?1", [id])?;
    Ok(())
}

#[tauri::command]
pub async fn scan_source(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    source_id: String,
    folder: Option<String>,
    platform_filter: Option<String>,
) -> Result<ScanResult> {
    state.cancel_scan.store(false, Ordering::SeqCst);
    let state = state.inner().clone();
    let cancel = state.cancel_scan.clone();
    tokio::task::spawn_blocking(move || {
        scanner::scan_source(
            &state,
            &source_id,
            Some(app),
            cancel,
            folder,
            platform_filter,
        )
    })
    .await
    .map_err(|e| RetroError::Other(e.to_string()))?
}

#[tauri::command]
pub async fn scan_local_folder(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    source_id: String,
    folder_path: String,
    platform_filter: Option<String>,
) -> Result<ScanResult> {
    state.cancel_scan.store(false, Ordering::SeqCst);
    let state = state.inner().clone();
    let cancel = state.cancel_scan.clone();
    tokio::task::spawn_blocking(move || {
        scanner::scan_source(
            &state,
            &source_id,
            Some(app),
            cancel,
            Some(folder_path),
            platform_filter,
        )
    })
    .await
    .map_err(|e| RetroError::Other(e.to_string()))?
}

#[tauri::command]
pub fn cancel_scan(state: State<AppState>) -> Result<()> {
    state.cancel_scan.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn update_source(
    state: State<AppState>,
    id: String,
    name: String,
    path: String,
) -> Result<GameSource> {
    let p = std::path::Path::new(&path);
    if !p.exists() || !p.is_dir() {
        return Err(RetroError::Invalid(
            "path must be an existing directory".into(),
        ));
    }
    if p.components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(RetroError::Invalid(
            "path must not contain '..' components".into(),
        ));
    }
    let conn = state.db.lock().unwrap();
    conn.execute(
        "UPDATE game_sources SET name=?1, path=?2 WHERE id=?3",
        params![name, path, id.clone()],
    )?;
    // return updated
    let src: GameSource = conn.query_row(
        "SELECT id,name,kind,path,created_at,last_scan_at,connection_id,remote_path,available,last_error,active FROM game_sources WHERE id=?1",
        [id.clone()],
        |r| {
            let created: String = r.get(4)?;
            let last: Option<String> = r.get(5)?;
            Ok(GameSource {
                id: r.get(0)?,
                name: r.get(1)?,
                kind: r.get(2)?,
                path: r.get(3)?,
                created_at: created.parse().unwrap_or_else(|_| Utc::now()),
                last_scan_at: last.and_then(|s| s.parse().ok()),
                connection_id: r.get(6)?,
                remote_path: r.get(7)?,
                available: r.get::<_, i64>(8).unwrap_or(1) != 0,
                last_error: r.get(9)?,
                active: r.get::<_, i64>(10).unwrap_or(0) != 0,
            })
        },
    )?;
    Ok(src)
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn update_remote_source(
    state: State<AppState>,
    id: String,
    name: String,
    host: String,
    port: i64,
    username: String,
    key_path: String,
    remote_path: String,
) -> Result<GameSource> {
    let conn = state.db.lock().unwrap();
    // update or create connection
    let cid: Option<String> = conn
        .query_row(
            "SELECT connection_id FROM game_sources WHERE id=?1",
            [id.clone()],
            |r| r.get(0),
        )
        .ok()
        .flatten();
    let _conn_id = match cid {
        Some(c) => {
            conn.execute("UPDATE remote_connections SET name=?1,host=?2,port=?3,username=?4,key_path=?5 WHERE id=?6", params![name, host, port, username, key_path, c])?;
            c
        }
        None => {
            let new_id = crate::models::new_id();
            let now = Utc::now().to_rfc3339();
            conn.execute("INSERT INTO remote_connections (id,name,host,port,username,key_path,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)", params![new_id, name, host, port, username, key_path, now])?;
            conn.execute(
                "UPDATE game_sources SET connection_id=?1 WHERE id=?2",
                params![new_id, id],
            )?;
            new_id
        }
    };
    conn.execute(
        "UPDATE game_sources SET name=?1, remote_path=?2, path=?2 WHERE id=?3",
        params![name, remote_path, id.clone()],
    )?;
    let src: GameSource = conn.query_row(
        "SELECT id,name,kind,path,created_at,last_scan_at,connection_id,remote_path,available,last_error,active FROM game_sources WHERE id=?1",
        [id.clone()],
        |r| {
            let created: String = r.get(4)?;
            let last: Option<String> = r.get(5)?;
            Ok(GameSource {
                id: r.get(0)?,
                name: r.get(1)?,
                kind: r.get(2)?,
                path: r.get(3)?,
                created_at: created.parse().unwrap_or_else(|_| Utc::now()),
                last_scan_at: last.and_then(|s| s.parse().ok()),
                connection_id: r.get(6)?,
                remote_path: r.get(7)?,
                available: r.get::<_, i64>(8).unwrap_or(1) != 0,
                last_error: r.get(9)?,
                active: r.get::<_, i64>(10).unwrap_or(0) != 0,
            })
        },
    )?;
    Ok(src)
}

#[tauri::command]
pub fn set_active_source(state: State<AppState>, id: String) -> Result<()> {
    let conn = state.db.lock().unwrap();
    // verify exists
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM game_sources WHERE id=?1",
            [id.clone()],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0)
        > 0;
    if !exists {
        return Err(RetroError::NotFound("source not found".into()));
    }
    // test accessibility: for local, check path exists; for remote, skip (assume ok)
    let kind: String = conn.query_row(
        "SELECT kind FROM game_sources WHERE id=?1",
        [id.clone()],
        |r| r.get(0),
    )?;
    if kind == "local" {
        let path: String = conn.query_row(
            "SELECT path FROM game_sources WHERE id=?1",
            [id.clone()],
            |r| r.get(0),
        )?;
        if !std::path::Path::new(&path).exists() {
            return Err(RetroError::Invalid(format!(
                "path does not exist: {}",
                path
            )));
        }
    }
    conn.execute("UPDATE game_sources SET active=0", [])?;
    conn.execute("UPDATE game_sources SET active=1 WHERE id=?1", [id])?;
    Ok(())
}

#[tauri::command]
pub fn get_active_source(state: State<AppState>) -> Result<Option<GameSource>> {
    let conn = state.db.lock().unwrap();
    let res = conn.query_row(
        "SELECT id,name,kind,path,created_at,last_scan_at,connection_id,remote_path,available,last_error,active FROM game_sources WHERE active=1 LIMIT 1",
        [],
        |r| {
            let created: String = r.get(4)?;
            let last: Option<String> = r.get(5)?;
            Ok(GameSource {
                id: r.get(0)?,
                name: r.get(1)?,
                kind: r.get(2)?,
                path: r.get(3)?,
                created_at: created.parse().unwrap_or_else(|_| Utc::now()),
                last_scan_at: last.and_then(|s| s.parse().ok()),
                connection_id: r.get(6)?,
                remote_path: r.get(7)?,
                available: r.get::<_, i64>(8).unwrap_or(1) != 0,
                last_error: r.get(9)?,
                active: true,
            })
        },
    );
    match res {
        Ok(s) => Ok(Some(s)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(RetroError::Db(e)),
    }
}

#[tauri::command]
pub fn test_source(state: State<AppState>, id: String) -> Result<String> {
    let conn = state.db.lock().unwrap();
    let kind: String = conn.query_row(
        "SELECT kind FROM game_sources WHERE id=?1",
        [id.clone()],
        |r| r.get(0),
    )?;
    if kind == "local" {
        let path: String =
            conn.query_row("SELECT path FROM game_sources WHERE id=?1", [id], |r| {
                r.get(0)
            })?;
        if std::path::Path::new(&path).exists() {
            Ok("Source locale accessible.".into())
        } else {
            Err(RetroError::Invalid(format!(
                "Dossier introuvable : {}",
                path
            )))
        }
    } else {
        // remote: check connection_id
        let cid: Option<String> = conn.query_row(
            "SELECT connection_id FROM game_sources WHERE id=?1",
            [id.clone()],
            |r| r.get(0),
        )?;
        let cid = cid.ok_or_else(|| RetroError::Invalid("pas de connexion associée".into()))?;
        let (host, port, user, key): (String, i64, String, String) = conn.query_row(
            "SELECT host,port,username,key_path FROM remote_connections WHERE id=?1",
            [cid],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?;
        drop(conn);
        let out = std::process::Command::new("ssh")
            .args([
                "-i",
                &key,
                "-p",
                &port.to_string(),
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=10",
                &format!("{}@{}", user, host),
                "echo OK",
            ])
            .output()
            .map_err(RetroError::Io)?;
        if out.status.success() {
            Ok("Connexion SSH réussie.".into())
        } else {
            Err(RetroError::Other(
                String::from_utf8_lossy(&out.stderr).to_string(),
            ))
        }
    }
}

#[tauri::command]
pub fn get_remote_connection(
    state: State<AppState>,
    source_id: String,
) -> Result<serde_json::Value> {
    let conn = state.db.lock().unwrap();
    let cid: Option<String> = conn
        .query_row(
            "SELECT connection_id FROM game_sources WHERE id=?1",
            [source_id],
            |r| r.get(0),
        )
        .ok()
        .flatten();
    let cid = cid.ok_or_else(|| RetroError::NotFound("no connection".into()))?;
    let (host, port, user, key): (String, i64, String, String) = conn.query_row(
        "SELECT host,port,username,key_path FROM remote_connections WHERE id=?1",
        [cid],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    )?;
    Ok(serde_json::json!({"host": host, "port": port, "username": user, "key_path": key}))
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn convert_source(
    state: State<'_, AppState>,
    id: String,
    new_kind: String,
    name: String,
    local_path: Option<String>,
    host: Option<String>,
    port: Option<i64>,
    username: Option<String>,
    key_path: Option<String>,
    remote_path: Option<String>,
) -> Result<GameSource> {
    // Validate before transaction.
    if new_kind == "local" {
        let p = local_path.clone().unwrap_or_default();
        let pp = std::path::Path::new(&p);
        if !pp.exists() || !pp.is_dir() {
            return Err(RetroError::Invalid(format!(
                "path does not exist or is not a directory: {}",
                p
            )));
        }
        if pp
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(RetroError::Invalid(
                "path must not contain '..' components".into(),
            ));
        }
    } else if new_kind == "remote" {
        let host = host.clone().unwrap_or_default();
        let port = port.unwrap_or(22);
        let username = username.clone().unwrap_or_default();
        let key_path = key_path.clone().unwrap_or_default();
        let remote_path = remote_path.clone().unwrap_or_default();
        if host.is_empty() || username.is_empty() || key_path.is_empty() || remote_path.is_empty() {
            return Err(RetroError::Invalid(
                "host, username, key and remote path are required".into(),
            ));
        }
        // Test SSH connection.
        let out = std::process::Command::new("ssh")
            .args([
                "-i",
                &key_path,
                "-p",
                &port.to_string(),
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=10",
                &format!("{}@{}", username, host),
                "test -d",
                &remote_path,
            ])
            .output()
            .map_err(RetroError::Io)?;
        if !out.status.success() {
            return Err(RetroError::Other(format!(
                "SSH connection failed: {}",
                String::from_utf8_lossy(&out.stderr)
            )));
        }
    } else {
        return Err(RetroError::Invalid(format!("unknown kind: {}", new_kind)));
    }

    let conn = state.db.lock().unwrap();
    let tx = conn.unchecked_transaction().map_err(RetroError::Db)?;

    // Get old kind and connection_id.
    let (_old_kind, old_conn_id): (String, Option<String>) = tx
        .query_row(
            "SELECT kind, connection_id FROM game_sources WHERE id=?1",
            [&id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(RetroError::Db)?;

    let now = chrono::Utc::now().to_rfc3339();

    if new_kind == "local" {
        let path = local_path.unwrap_or_default();
        // Remove old remote connection if any.
        if let Some(cid) = old_conn_id {
            let _ = tx.execute("DELETE FROM remote_connections WHERE id=?1", [cid]);
        }
        tx.execute(
            "UPDATE game_sources SET name=?1, kind='local', path=?2, connection_id=NULL, remote_path=NULL, available=1, last_error=NULL WHERE id=?3",
            params![name, path, id],
        ).map_err(RetroError::Db)?;
    } else {
        let host = host.unwrap_or_default();
        let port = port.unwrap_or(22);
        let username = username.unwrap_or_default();
        let key_path = key_path.unwrap_or_default();
        let remote_path = remote_path.unwrap_or_default();
        // Create or update remote connection.
        let conn_id = match old_conn_id {
            Some(cid) => {
                tx.execute("UPDATE remote_connections SET name=?1, host=?2, port=?3, username=?4, key_path=?5 WHERE id=?6", params![name, host, port, username, key_path, cid])?;
                cid
            }
            None => {
                let new_id = crate::models::new_id();
                tx.execute("INSERT INTO remote_connections (id,name,host,port,username,key_path,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)", params![new_id, name, host, port, username, key_path, now])?;
                new_id
            }
        };
        tx.execute(
            "UPDATE game_sources SET name=?1, kind='remote', path=?2, connection_id=?3, remote_path=?2, available=1, last_error=NULL WHERE id=?4",
            params![name, remote_path, conn_id, id],
        ).map_err(RetroError::Db)?;
    }

    tx.commit().map_err(RetroError::Db)?;

    // Return updated source.
    let conn = state.db.lock().unwrap();
    let src: GameSource = conn.query_row(
        "SELECT id,name,kind,path,created_at,last_scan_at,connection_id,remote_path,available,last_error,active FROM game_sources WHERE id=?1",
        [id],
        |r| {
            let created: String = r.get(4)?;
            let last: Option<String> = r.get(5)?;
            Ok(GameSource {
                id: r.get(0)?,
                name: r.get(1)?,
                kind: r.get(2)?,
                path: r.get(3)?,
                created_at: created.parse().unwrap_or_else(|_| chrono::Utc::now()),
                last_scan_at: last.and_then(|s| s.parse().ok()),
                connection_id: r.get(6)?,
                remote_path: r.get(7)?,
                available: r.get::<_, i64>(8).unwrap_or(1) != 0,
                last_error: r.get(9)?,
                active: r.get::<_, i64>(10).unwrap_or(0) != 0,
            })
        },
    )?;
    Ok(src)
}
