use crate::error::{Result, RetroError};
use crate::state::AppState;
use rusqlite::params;
use serde_json::json;
use tauri::State;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct GameListEntry {
    path: String,
    name: String,
    desc: Option<String>,
    developer: Option<String>,
    publisher: Option<String>,
    genre: Option<String>,
    players: Option<String>,
    rating: Option<String>,
    releasedate: Option<String>,
}

fn parse_gamelist(content: &str) -> Vec<GameListEntry> {
    let mut out = Vec::new();
    // split into <game>...</game>
    let mut rest = content;
    while let Some(s) = rest.find("<game>") {
        rest = &rest[s + 6..];
        if let Some(e) = rest.find("</game>") {
            let block = &rest[..e];
            rest = &rest[e + 7..];
            let get = |tag: &str| {
                let open = format!("<{}>", tag);
                let close = format!("</{}>", tag);
                if let Some(i) = block.find(&open) {
                    let after = &block[i + open.len()..];
                    if let Some(j) = after.find(&close) {
                        return Some(after[..j].trim().to_string());
                    }
                }
                None
            };
            let path = match get("path") {
                Some(p) if !p.is_empty() => p,
                _ => continue,
            };
            let name = match get("name") {
                Some(n) if !n.is_empty() => n,
                _ => continue,
            };
            out.push(GameListEntry {
                path,
                name,
                desc: get("desc"),
                developer: get("developer"),
                publisher: get("publisher"),
                genre: get("genre"),
                players: get("players"),
                rating: get("rating"),
                releasedate: get("releasedate"),
            });
        } else {
            break;
        }
    }
    out
}

fn emulation_root_from_source_path(path: &str) -> String {
    // source path is usually .../roms or .../roms/<platform>; go up to Emulation
    let p = std::path::Path::new(path);
    // if ends with roms, parent is Emulation
    if p.ends_with("roms") {
        return p.parent().unwrap_or(p).to_string_lossy().to_string();
    }
    // if parent ends with roms, go up two
    if let Some(parent) = p.parent()
        && parent.ends_with("roms")
    {
        return parent.parent().unwrap_or(parent).to_string_lossy().to_string();
    }
    // fallback: parent
    p.parent().unwrap_or(p).to_string_lossy().to_string()
}

#[tauri::command]
pub async fn import_gamelists(state: State<'_, AppState>, source_id: String) -> Result<usize> {
    // get source info
    let (kind, path, remote_path, connection_id) = {
        let conn = state.db.lock().unwrap();
        let (kind, path, rp, cid): (String, String, Option<String>, Option<String>) = conn.query_row(
            "SELECT kind, path, remote_path, connection_id FROM game_sources WHERE id=?1",
            [source_id.clone()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?;
        (kind, path, rp, cid)
    };

    let base_path = remote_path.clone().unwrap_or_else(|| path.clone());
    let emu_root = emulation_root_from_source_path(&base_path);

    // Collect gamelist contents (local or remote)
    let mut gamelists: Vec<(String, String)> = Vec::new(); // (platform, content)

    if kind == "remote" {
        let cid = connection_id.ok_or_else(|| RetroError::Invalid("not remote".into()))?;
        let (host, port, username, key_path) = {
            let conn = state.db.lock().unwrap();
            conn.query_row(
                "SELECT host,port,username,key_path FROM remote_connections WHERE id=?1",
                [cid],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u16, r.get::<_, String>(2)?, r.get::<_, String>(3)?)),
            )?
        };
        let sess = crate::remote::connect(&host, port, &username, &key_path).await?;
        // list gamelists dir
        let gl_dir = format!("{}/saves/es-de/gamelists", emu_root);
        if let Ok(rd) = sess.sftp.read_dir(&gl_dir).await {
            let dirs: Vec<_> = rd.collect();
            for d in dirs {
                if !d.metadata().is_dir() { continue; }
                let platform = d.file_name();
                let gp = format!("{}/saves/es-de/gamelists/{}/gamelist.xml", emu_root, platform);
                if let Ok(mut f) = sess.sftp.open(&gp).await {
                    use tokio::io::AsyncReadExt;
                    let mut buf = Vec::new();
                    let _ = f.read_to_end(&mut buf).await;
                    if let Ok(content) = String::from_utf8(buf) {
                        gamelists.push((platform, content));
                    }
                }
            }
        }
        let _ = sess.session.disconnect(russh::Disconnect::ByApplication, "", "").await;
    } else {
        let gl_dir = std::path::Path::new(&emu_root).join("saves").join("es-de").join("gamelists");
        if gl_dir.is_dir()
            && let Ok(rd) = std::fs::read_dir(&gl_dir)
        {
            for e in rd.flatten() {
                if !e.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }
                let platform = e.file_name().to_string_lossy().to_string();
                let gp = e.path().join("gamelist.xml");
                if gp.is_file()
                    && let Ok(content) = std::fs::read_to_string(&gp)
                {
                    gamelists.push((platform, content));
                }
            }
        }
    }

    // Import
    let mut imported = 0usize;
    let conn = state.db.lock().unwrap();
    for (platform, content) in &gamelists {
        let entries = parse_gamelist(content);
        for e in entries {
            // find platform id by slug
            let platform_id: Option<String> = conn.query_row(
                "SELECT id FROM platforms WHERE slug=?1 OR id=?1",
                [platform.clone()],
                |r| r.get(0),
            ).ok();

            // find or create game
            let existing_game: Option<String> = conn.query_row(
                "SELECT id FROM games WHERE LOWER(title)=LOWER(?1) AND (platform_id IS ?2 OR platform_id=?2)",
                params![e.name, platform_id],
                |r| r.get(0),
            ).ok();

            let game_id = if let Some(gid) = existing_game {
                gid
            } else {
                let gid = crate::models::new_id();
                conn.execute(
                    "INSERT INTO games (id,title,platform_id,created_at,updated_at) VALUES (?1,?2,?3,?4,?5)",
                    params![gid, e.name, platform_id, chrono::Utc::now().to_rfc3339(), chrono::Utc::now().to_rfc3339()],
                )?;
                imported += 1;
                gid
            };

            // store metadata
            let now = chrono::Utc::now().to_rfc3339();
            if let Some(desc) = &e.desc {
                let id = crate::models::new_id();
                conn.execute(
                    "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![id, game_id, "es-de", "description", desc, "es-de", now],
                )?;
            }
            if let Some(dev) = &e.developer {
                let id = crate::models::new_id();
                conn.execute(
                    "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![id, game_id, "es-de", "developer", dev, "es-de", now],
                )?;
            }
            if let Some(pub_) = &e.publisher {
                let id = crate::models::new_id();
                conn.execute(
                    "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![id, game_id, "es-de", "publisher", pub_, "es-de", now],
                )?;
            }
            if let Some(genre) = &e.genre {
                let id = crate::models::new_id();
                conn.execute(
                    "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![id, game_id, "es-de", "genre", genre, "es-de", now],
                )?;
            }
            if let Some(rd) = &e.releasedate {
                let id = crate::models::new_id();
                conn.execute(
                    "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![id, game_id, "es-de", "release_date", rd, "es-de", now],
                )?;
            }
            let _ = json!({});
        }
    }

    Ok(imported)
}