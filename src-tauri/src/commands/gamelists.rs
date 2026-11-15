//! Commands for gamelists.

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
        return parent
            .parent()
            .unwrap_or(parent)
            .to_string_lossy()
            .to_string();
    }
    // fallback: parent
    p.parent().unwrap_or(p).to_string_lossy().to_string()
}

pub async fn import_gamelists_inner(state: &AppState, source_id: String) -> Result<usize> {
    // get source info
    let (kind, path, remote_path, connection_id) = {
        let conn = state.db.lock().unwrap();
        let (kind, path, rp, cid): (String, String, Option<String>, Option<String>) = conn
            .query_row(
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
        // list gamelists dir
        let gl_dir = format!("{}/saves/es-de/gamelists", emu_root);
        if let Ok(rd) = sess.sftp.read_dir(&gl_dir).await {
            let dirs: Vec<_> = rd.collect();
            for d in dirs {
                if !d.metadata().is_dir() {
                    continue;
                }
                let platform = d.file_name();
                let gp = format!(
                    "{}/saves/es-de/gamelists/{}/gamelist.xml",
                    emu_root, platform
                );
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

        let _ = sess
            .session
            .disconnect(russh::Disconnect::ByApplication, "", "")
            .await;
    } else {
        let gl_dir = std::path::Path::new(&emu_root)
            .join("saves")
            .join("es-de")
            .join("gamelists");
        if gl_dir.is_dir()
            && let Ok(rd) = std::fs::read_dir(&gl_dir)
        {
            for e in rd.flatten() {
                if !e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
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

    // Import (optimized with in-memory game lookup + transaction)
    let mut imported = 0usize;
    let mut conn = state.db.lock().unwrap();
    let tx = conn.transaction()?;

    // Pre-load existing games into a map for O(1) lookup.
    let mut existing_games: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    {
        let mut stmt = tx.prepare("SELECT id, title, platform_id FROM games")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        })?;
        for r in rows {
            let (id, title, pid) = r?;
            existing_games.insert(
                format!("{}|{}", title.to_lowercase(), pid.unwrap_or_default()),
                id,
            );
        }
    }
    // Pre-load existing metadata records into a set for O(1) lookup.
    let mut existing_meta: std::collections::HashSet<String> = std::collections::HashSet::new();
    {
        let mut stmt = tx.prepare("SELECT game_id, field FROM metadata_records")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        for r in rows {
            let (gid, field) = r?;
            existing_meta.insert(format!("{}|{}", gid, field));
        }
    }

    for (platform, content) in &gamelists {
        let entries = parse_gamelist(content);
        let platform_id: Option<String> = tx
            .query_row(
                "SELECT id FROM platforms WHERE slug=?1 OR id=?1",
                [platform.clone()],
                |r| r.get(0),
            )
            .ok();

        for e in &entries {
            let key = format!(
                "{}|{}",
                e.name.to_lowercase(),
                platform_id.clone().unwrap_or_default()
            );
            let game_id = if let Some(gid) = existing_games.get(&key) {
                gid.clone()
            } else {
                let gid = crate::models::new_id();
                tx.execute(
                    "INSERT INTO games (id,title,platform_id,created_at,updated_at) VALUES (?1,?2,?3,?4,?5)",
                    params![gid, e.name, platform_id, chrono::Utc::now().to_rfc3339(), chrono::Utc::now().to_rfc3339()],
                )?;
                existing_games.insert(key, gid.clone());
                imported += 1;
                gid
            };

            let now = chrono::Utc::now().to_rfc3339();
            let fields: Vec<(&str, Option<&String>)> = vec![
                ("description", e.desc.as_ref()),
                ("developer", e.developer.as_ref()),
                ("publisher", e.publisher.as_ref()),
                ("genre", e.genre.as_ref()),
                ("release_date", e.releasedate.as_ref()),
            ];
            for (field, val) in fields {
                if let Some(v) = val {
                    let key = format!("{}|{}", game_id, field);
                    if !existing_meta.contains(&key) {
                        let id = crate::models::new_id();
                        tx.execute(
                            "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                            params![id, game_id, "es-de", field, v, "es-de", now],
                        )?;
                        existing_meta.insert(key);
                    }
                }
            }
            let _ = json!({});
        }
    }

    tx.commit()?;
    Ok(imported)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn meta_value(conn: &rusqlite::Connection, game_id: &str, field: &str) -> Option<String> {
    conn
        .query_row(
            "SELECT value FROM metadata_records WHERE game_id=?1 AND field=?2 ORDER BY fetched_at DESC LIMIT 1",
            rusqlite::params![game_id, field],
            |r| r.get(0),
        )
        .ok()
}

pub async fn generate_gamelist_inner(
    state: &AppState,
    source_id: String,
    platform_filter: Option<String>,
) -> Result<usize> {
    // source info
    let (kind, path) = {
        let conn = state.db.lock().unwrap();
        let (kind, path): (String, String) = conn.query_row(
            "SELECT kind, path FROM game_sources WHERE id=?1",
            [source_id.clone()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        (kind, path)
    };
    if kind != "local" {
        return Err(RetroError::Invalid(
            "generate gamelist only for local sources".into(),
        ));
    }
    let emu_root = emulation_root_from_source_path(&path);
    let root = std::path::Path::new(&emu_root);
    let gl_root = root.join("saves").join("es-de").join("gamelists");

    // Determine which platforms to generate for.
    let platforms: Vec<(String, String)> = {
        let conn = state.db.lock().unwrap();
        if let Some(pf) = &platform_filter {
            // resolve slug/id
            let pid: Option<String> = conn
                .query_row(
                    "SELECT id FROM platforms WHERE id=?1 OR slug=?1",
                    [pf.clone()],
                    |r| r.get(0),
                )
                .ok();
            if let Some(pid) = pid {
                let slug: String = conn
                    .query_row(
                        "SELECT slug FROM platforms WHERE id=?1",
                        [pid.clone()],
                        |r| r.get(0),
                    )
                    .unwrap_or_else(|_| pid.clone());
                vec![(pid, slug)]
            } else {
                Vec::new()
            }
        } else {
            // distinct platforms present in this source
            let mut stmt = conn.prepare(
                "SELECT DISTINCT gf.platform_id FROM game_files gf WHERE gf.source_id=?1 AND gf.platform_id IS NOT NULL",
            )?;
            let rows = stmt.query_map([source_id.clone()], |r| r.get::<_, String>(0))?;
            let mut out = Vec::new();
            for r in rows {
                let pid = r?;
                let slug: String = conn
                    .query_row(
                        "SELECT slug FROM platforms WHERE id=?1",
                        [pid.clone()],
                        |r| r.get(0),
                    )
                    .unwrap_or_else(|_| pid.clone());
                out.push((pid, slug));
            }
            out
        }
    };

    let mut total_games = 0usize;
    for (platform_id, slug) in &platforms {
        // Collect games for this platform in this source (dedup by game).
        let game_ids: Vec<String> = {
            let conn = state.db.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT DISTINCT g.id FROM games g JOIN game_file_links l ON l.game_id=g.id JOIN game_files gf ON gf.id=l.file_id WHERE gf.source_id=?1 AND g.platform_id=?2",
            )?;
            let rows = stmt.query_map(rusqlite::params![source_id, platform_id], |r| r.get(0))?;
            rows.filter_map(|r| r.ok()).collect()
        };

        if game_ids.is_empty() {
            continue;
        }

        let dir = gl_root.join(slug);
        let _ = std::fs::create_dir_all(&dir);
        let dest = dir.join("gamelist.xml");

        let mut content = String::new();
        content.push_str("<?xml version=\"1.0\"?>\n");
        content.push_str("<gameList>\n");

        for gid in &game_ids {
            let conn = state.db.lock().unwrap();
            // title
            let title: String = conn
                .query_row("SELECT title FROM games WHERE id=?1", [gid.clone()], |r| {
                    r.get(0)
                })
                .unwrap_or_default();
            // a representative file path
            let file_path: Option<String> = conn
                .query_row(
                    "SELECT gf.path FROM game_files gf JOIN game_file_links l ON l.file_id=gf.id WHERE l.game_id=?1 AND gf.source_id=?2 LIMIT 1",
                    rusqlite::params![gid, source_id],
                    |r| r.get(0),
                )
                .ok();
            let filename = file_path
                .as_ref()
                .and_then(|p| std::path::Path::new(p).file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let rel_path = if filename.is_empty() {
                format!("./{}", title)
            } else {
                format!("./{}", filename)
            };

            let desc = meta_value(&conn, gid, "description");
            let dev = meta_value(&conn, gid, "developer");
            let pubv = meta_value(&conn, gid, "publisher");
            let genre = meta_value(&conn, gid, "genre");
            let rd = meta_value(&conn, gid, "release_date");

            content.push_str("  <game>\n");
            content.push_str(&format!("    <path>{}</path>\n", xml_escape(&rel_path)));
            content.push_str(&format!("    <name>{}</name>\n", xml_escape(&title)));
            if let Some(d) = &desc {
                content.push_str(&format!("    <desc>{}</desc>\n", xml_escape(d)));
            }
            if let Some(d) = &dev {
                content.push_str(&format!("    <developer>{}</developer>\n", xml_escape(d)));
            }
            if let Some(p) = &pubv {
                content.push_str(&format!("    <publisher>{}</publisher>\n", xml_escape(p)));
            }
            if let Some(g) = &genre {
                content.push_str(&format!("    <genre>{}</genre>\n", xml_escape(g)));
            }
            if let Some(r) = &rd {
                content.push_str(&format!(
                    "    <releasedate>{}</releasedate>\n",
                    xml_escape(r)
                ));
            }
            content.push_str("  </game>\n");
            total_games += 1;
        }

        content.push_str("</gameList>\n");
        std::fs::write(&dest, content).map_err(RetroError::Io)?;
    }

    Ok(total_games)
}

#[tauri::command]
pub async fn generate_gamelist(
    state: State<'_, AppState>,
    source_id: String,
    platform_id: Option<String>,
) -> Result<usize> {
    generate_gamelist_inner(&state, source_id, platform_id).await
}
#[tauri::command]
pub async fn import_gamelists(state: State<'_, AppState>, source_id: String) -> Result<usize> {
    import_gamelists_inner(&state, source_id).await
}
