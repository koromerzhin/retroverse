use crate::error::{Result, RetroError};
use crate::metadata;
use crate::state::AppState;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn list_metadata_providers() -> Vec<String> {
    metadata::providers_with_config(&crate::config::load())
        .iter()
        .map(|p| p.name().to_string())
        .collect()
}

#[tauri::command]
pub async fn enrich_game(
    state: State<'_, AppState>,
    game_id: String,
    provider: String,
) -> Result<serde_json::Value> {
    // get game title
    let (title, platform): (String, Option<String>) = {
        let conn = state.db.lock().unwrap();
        conn.query_row(
            "SELECT title, platform_id FROM games WHERE id=?1",
            [game_id.clone()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?
    };

    // Gather file hashes for this game (if available) to improve ScreenScraper matching.
    let hashes: Option<(Option<String>, Option<String>, Option<String>)> = {
        let conn = state.db.lock().unwrap();
        conn.query_row(
            "SELECT crc32, md5, sha1 FROM game_files gf JOIN game_file_links l ON l.file_id = gf.id WHERE l.game_id = ?1 LIMIT 1",
            [game_id.clone()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        ).ok()
    };

    let prov = metadata::provider_by_name_with_config(&provider, &crate::config::load())
        .ok_or_else(|| RetroError::Invalid("unknown provider".into()))?;

    let res = prov.fetch(&title, platform.as_deref(), hashes)?;

    // store metadata records with provenance
    {
        let conn = state.db.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        if let Some(desc) = &res.description {
            let exists: bool = conn.query_row(
                "SELECT COUNT(*) FROM metadata_records WHERE game_id=?1 AND field='description'",
                [game_id.clone()],
                |r| r.get::<_, i64>(0),
            ).unwrap_or(0) > 0;
            if !exists {
                let id = crate::models::new_id();
                conn.execute(
                    "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![id, game_id, provider, "description", desc, provider, now],
                )?;
            }
        }
        if let Some(dev) = &res.developer {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM metadata_records WHERE game_id=?1 AND field='developer'",
                    [game_id.clone()],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap_or(0)
                > 0;
            if !exists {
                let id = crate::models::new_id();
                conn.execute(
                    "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![id, game_id, provider, "developer", dev, provider, now],
                )?;
            }
        }
        if let Some(cov) = &res.cover_url {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM media_assets WHERE game_id=?1 AND kind='cover'",
                    [game_id.clone()],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap_or(0)
                > 0;
            if !exists {
                let id = crate::models::new_id();
                conn.execute(
                    "INSERT INTO media_assets (id,game_id,kind,url,cached_at) VALUES (?1,?2,?3,?4,?5)",
                    params![id, game_id, "cover", cov, now],
                )?;
            }
        }
        if let Some(t) = &res.title
            && t != &title
        {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM metadata_records WHERE game_id=?1 AND field='alt_title'",
                    [game_id.clone()],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap_or(0)
                > 0;
            if !exists {
                // don't overwrite title automatically; store as alternative
                let id = crate::models::new_id();
                conn.execute(
                    "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![id, game_id, provider, "alt_title", t, provider, now],
                )?;
            }
        }
        // store the full ScreenScraper game object as JSON
        {
            let details = res.raw.clone().unwrap_or_else(|| {
                serde_json::json!({
                    "ss_id": res.ss_id,
                    "notgame": res.notgame,
                    "systeme": res.systeme,
                    "editeur": res.publisher,
                    "developpeur": res.developer,
                    "joueurs": res.players,
                    "note": res.rating,
                    "topstaff": res.topstaff,
                    "rotation": res.rotation,
                    "synopsis": res.description,
                    "dates": res.release_dates,
                    "classifications": res.classifications,
                    "genres": res.genres,
                    "alt_titles": res.alt_titles,
                    "modes": res.modes,
                    "themes": res.themes,
                    "styles": res.styles,
                    "familles": res.familles,
                })
            });
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM metadata_records WHERE game_id=?1 AND field='details'",
                    [game_id.clone()],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap_or(0)
                > 0;
            if !exists {
                let id = crate::models::new_id();
                conn.execute(
                    "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![id, game_id, provider, "details", details.to_string(), provider, now],
                )?;
            }
        }

        // also store details in game.metadata_json (prefer full raw ScreenScraper object)
        {
            let details = res.raw.clone().unwrap_or_else(|| {
                serde_json::json!({
                    "publisher": res.publisher,
                    "developer": res.developer,
                    "genres": res.genres,
                    "players": res.players,
                    "rating": res.rating,
                    "release_dates": res.release_dates,
                    "classifications": res.classifications,
                    "alt_titles": res.alt_titles,
                    "modes": res.modes,
                    "themes": res.themes,
                    "styles": res.styles,
                    "ss_id": res.ss_id,
                    "notgame": res.notgame,
                    "systeme": res.systeme,
                    "topstaff": res.topstaff,
                    "rotation": res.rotation,
                    "synopsis": res.description,
                    "familles": res.familles,
                })
            });
            let _ = conn.execute(
                "UPDATE games SET metadata_json=?1 WHERE id=?2",
                params![details.to_string(), game_id.clone()],
            );
        }

        // record job
        let job_id = crate::models::new_id();
        conn.execute(
            "INSERT INTO metadata_jobs (id,game_id,provider,status,created_at,finished_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![job_id, game_id, provider, "completed", now, now],
        )?;
    }

    Ok(serde_json::json!({
        "provider": provider,
        "has_description": res.description.is_some(),
        "has_cover": res.cover_url.is_some(),
    }))
}

#[tauri::command]
pub fn get_game_metadata(
    state: State<AppState>,
    game_id: String,
) -> Result<Vec<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT provider,field,value,provenance,fetched_at FROM metadata_records WHERE game_id=?1 ORDER BY fetched_at DESC")?;
    let rows = stmt.query_map([game_id.clone()], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, String>(4)?,
        ))
    })?;
    let mut seen_fields: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out = Vec::new();
    for r in rows {
        let (provider, field, value, provenance, fetched_at) = r?;
        if !seen_fields.insert(field.clone()) {
            continue;
        }
        out.push(serde_json::json!({
            "provider": provider,
            "field": field,
            "value": value,
            "provenance": provenance,
            "fetched_at": fetched_at,
        }));
    }
    // media: deduplicate by kind
    let mut mstmt =
        conn.prepare("SELECT kind,url FROM media_assets WHERE game_id=?1 ORDER BY cached_at DESC")?;
    let mrows = mstmt.query_map([game_id.clone()], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
    })?;
    let mut seen_kinds: std::collections::HashSet<String> = std::collections::HashSet::new();
    for r in mrows {
        let (kind, url) = r?;
        if !seen_kinds.insert(kind.clone()) {
            continue;
        }
        out.push(serde_json::json!({"kind": kind, "url": url}));
    }
    Ok(out)
}

#[tauri::command]
pub fn list_developers(state: State<AppState>) -> Result<Vec<String>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT DISTINCT value FROM metadata_records WHERE field='developer' AND value IS NOT NULL AND value != '' ORDER BY value COLLATE NOCASE")?;
    let rows = stmt.query_map([], |r| r.get(0))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[tauri::command]
pub fn list_publishers(state: State<AppState>) -> Result<Vec<String>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT DISTINCT value FROM metadata_records WHERE field='publisher' AND value IS NOT NULL AND value != '' ORDER BY value COLLATE NOCASE")?;
    let rows = stmt.query_map([], |r| r.get(0))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[tauri::command]
pub fn set_game_metadata(
    state: State<AppState>,
    game_id: String,
    field: String,
    value: String,
) -> Result<()> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "DELETE FROM metadata_records WHERE game_id=?1 AND field=?2",
        params![game_id.clone(), field.clone()],
    )?;
    if !value.trim().is_empty() {
        let id = crate::models::new_id();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![id, game_id, "manual", field, value, "manual", now],
        )?;
    }
    Ok(())
}

#[tauri::command]
pub fn save_game_info(
    state: State<AppState>,
    game_id: String,
    title: String,
    metadata_json: String,
) -> Result<()> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "UPDATE games SET title=?1 WHERE id=?2",
        params![title, game_id.clone()],
    )?;
    conn.execute(
        "UPDATE games SET metadata_json=?1 WHERE id=?2",
        params![metadata_json, game_id.clone()],
    )?;
    // Upsert publisher/developer into referentials and link to the game.
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&metadata_json) {
        let now = chrono::Utc::now().to_rfc3339();
        // publisher
        let pub_name = v
            .get("editeur")
            .and_then(|e| e.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                v.get("publisher")
                    .and_then(|p| p.as_str())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            });
        if let Some(name) = pub_name {
            let pid: Option<String> = conn
                .query_row(
                    "SELECT id FROM publishers WHERE name=?1 COLLATE NOCASE",
                    [name.clone()],
                    |r| r.get(0),
                )
                .ok();
            let pid = match pid {
                Some(id) => id,
                None => {
                    let id = crate::models::new_id();
                    let _ = conn.execute(
                        "INSERT INTO publishers (id,name,ss_id,created_at,updated_at) VALUES (?1,?2,NULL,?3,?4)",
                        params![id, name, now, now],
                    );
                    id
                }
            };
            let _ = conn.execute(
                "INSERT OR IGNORE INTO game_publishers (game_id, publisher_id) VALUES (?1,?2)",
                params![game_id, pid],
            );
        }
        // developer
        let dev_name = v
            .get("developpeur")
            .and_then(|e| e.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                v.get("developer")
                    .and_then(|p| p.as_str())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            });
        if let Some(name) = dev_name {
            let did: Option<String> = conn
                .query_row(
                    "SELECT id FROM developers WHERE name=?1 COLLATE NOCASE",
                    [name.clone()],
                    |r| r.get(0),
                )
                .ok();
            let did = match did {
                Some(id) => id,
                None => {
                    let id = crate::models::new_id();
                    let _ = conn.execute(
                        "INSERT INTO developers (id,name,ss_id,created_at,updated_at) VALUES (?1,?2,NULL,?3,?4)",
                        params![id, name, now, now],
                    );
                    id
                }
            };
            let _ = conn.execute(
                "INSERT OR IGNORE INTO game_developers (game_id, developer_id) VALUES (?1,?2)",
                params![game_id, did],
            );
        }
    }
    Ok(())
}
