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

pub async fn enrich_game_inner(
    state: &AppState,
    game_id: String,
    provider: String,
) -> Result<serde_json::Value> {
    // state is &AppState now; adjust internal state.db -> state.db

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

    let res = match prov.fetch(&title, platform.as_deref(), hashes) {
        Ok(r) => r,
        Err(e) => {
            // Record that a search was performed with no result, so the UI can disable the button permanently.
            if matches!(e, RetroError::NotFound(_)) {
                let conn = state.db.lock().unwrap();
                let now = chrono::Utc::now().to_rfc3339();
                let exists: bool = conn.query_row(
                    "SELECT COUNT(*) FROM metadata_records WHERE game_id=?1 AND field='ss_no_result'",
                    [game_id.clone()],
                    |r| r.get::<_, i64>(0),
                ).unwrap_or(0) > 0;
                if !exists {
                    let id = crate::models::new_id();
                    let _ = conn.execute(
                        "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                        params![id, game_id.clone(), provider, "ss_no_result", "1", provider, now],
                    );
                }
            }
            return Err(e);
        }
    };

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
        if let Some(pubv) = &res.publisher {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM metadata_records WHERE game_id=?1 AND field='publisher'",
                    [game_id.clone()],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap_or(0)
                > 0;
            if !exists {
                let id = crate::models::new_id();
                conn.execute(
                    "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![id, game_id, provider, "publisher", pubv, provider, now],
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
                // Cache the cover image locally for offline use.
                let cached_path = (|| -> Option<String> {
                    let base = dirs::data_dir()?
                        .join("retroverse")
                        .join("cache")
                        .join("covers");
                    let _ = std::fs::create_dir_all(&base);
                    let dest = base.join(format!("{}.png", game_id));
                    // Download via ureq
                    let resp = ureq::get(cov).call().ok()?;
                    let mut r = resp;
                    let mut bytes = Vec::new();
                    use std::io::Read;
                    r.body_mut().as_reader().read_to_end(&mut bytes).ok()?;
                    std::fs::write(&dest, &bytes).ok()?;
                    Some(dest.to_string_lossy().to_string())
                })();
                let id = crate::models::new_id();
                conn.execute(
                    "INSERT INTO media_assets (id,game_id,kind,url,path,cached_at) VALUES (?1,?2,?3,?4,?5,?6)",
                    params![id, game_id, "cover", cov, cached_path, now],
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

        // Associate classifications with the game via the referential.
        for (sys_name, value) in &res.classifications {
            if sys_name.trim().is_empty() || value.trim().is_empty() {
                continue;
            }
            let sys_id: String = match conn.query_row(
                "SELECT id FROM classification_systems WHERE name=?1",
                [sys_name.clone()],
                |r| r.get(0),
            ) {
                Ok(id) => id,
                Err(_) => {
                    let id = crate::models::new_id();
                    let _ = conn.execute(
                        "INSERT INTO classification_systems (id,name,ss_id,created_at,updated_at) VALUES (?1,?2,NULL,?3,?4)",
                        params![id, sys_name, now, now],
                    );
                    id
                }
            };
            let cls_id: String = match conn.query_row(
                "SELECT id FROM classifications WHERE system_id=?1 AND value=?2",
                params![sys_id, value],
                |r| r.get(0),
            ) {
                Ok(id) => id,
                Err(_) => {
                    let id = crate::models::new_id();
                    let _ = conn.execute(
                        "INSERT INTO classifications (id,system_id,value,ss_id,description,created_at,updated_at) VALUES (?1,?2,?3,NULL,NULL,?4,?5)",
                        params![id, sys_id, value, now, now],
                    );
                    id
                }
            };
            let _ = conn.execute(
                "INSERT OR IGNORE INTO game_classifications (game_id, classification_id) VALUES (?1,?2)",
                params![game_id, cls_id],
            );
        }

        // record job
        let job_id = crate::models::new_id();
        conn.execute(
            "INSERT INTO metadata_jobs (id,game_id,provider,status,created_at,finished_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![job_id, game_id, provider, "completed", now, now],
        )?;
    }

    // If no useful data was retrieved, record ss_no_result so the UI can disable the button.
    if res.description.is_none() && res.cover_url.is_none() {
        let conn = state.db.lock().unwrap();
        let now2 = chrono::Utc::now().to_rfc3339();
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM metadata_records WHERE game_id=?1 AND field='ss_no_result'",
                [game_id.clone()],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;
        if !exists {
            let id = crate::models::new_id();
            let _ = conn.execute(
                "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                params![id, game_id.clone(), provider, "ss_no_result", "1", provider, now2],
            );
        }
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
    let mut mstmt = conn.prepare(
        "SELECT kind,url,path FROM media_assets WHERE game_id=?1 ORDER BY cached_at DESC",
    )?;
    let mrows = mstmt.query_map([game_id.clone()], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, Option<String>>(2)?,
        ))
    })?;
    let mut seen_kinds: std::collections::HashSet<String> = std::collections::HashSet::new();
    for r in mrows {
        let (kind, url, path) = r?;
        if !seen_kinds.insert(kind.clone()) {
            continue;
        }
        out.push(serde_json::json!({"kind": kind, "url": url, "path": path}));
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

#[tauri::command]
pub async fn enrich_game(
    state: State<'_, AppState>,
    game_id: String,
    provider: String,
) -> Result<serde_json::Value> {
    enrich_game_inner(&state, game_id, provider).await
}

#[tauri::command]
pub fn is_game_eligible(state: State<AppState>, game_id: String) -> Result<bool> {
    let conn = state.db.lock().unwrap();
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM game_files gf JOIN game_file_links l ON l.file_id=gf.id WHERE l.game_id=?1",
        [game_id.clone()],
        |r| r.get(0),
    ).unwrap_or(0);
    if total == 0 {
        return Ok(false);
    }
    let unhashed: i64 = conn.query_row(
        "SELECT COUNT(*) FROM game_files gf JOIN game_file_links l ON l.file_id=gf.id WHERE l.game_id=?1 AND (gf.hash IS NULL OR gf.crc32 IS NULL OR gf.md5 IS NULL OR gf.sha1 IS NULL OR gf.sha512 IS NULL OR ((gf.extension='zip' OR gf.extension='7z') AND gf.archive_hash IS NULL))",
        [game_id],
        |r| r.get(0),
    ).unwrap_or(0);
    Ok(unhashed == 0)
}

#[tauri::command]
pub async fn enrich_all_eligible(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<usize> {
    use tauri::Emitter;
    let game_ids: Vec<String> = {
        let conn = state.db.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id FROM games")?;
        let rows = stmt.query_map([], |r| r.get(0))?;
        rows.filter_map(|r| r.ok()).collect()
    };
    // Determine eligibility.
    let mut eligible: Vec<String> = Vec::new();
    let mut ignored = 0usize;
    for gid in &game_ids {
        let conn = state.db.lock().unwrap();
        let total: i64 = conn.query_row("SELECT COUNT(*) FROM game_files gf JOIN game_file_links l ON l.file_id=gf.id WHERE l.game_id=?1", [gid.clone()], |r| r.get(0)).unwrap_or(0);
        let unhashed: i64 = conn.query_row("SELECT COUNT(*) FROM game_files gf JOIN game_file_links l ON l.file_id=gf.id WHERE l.game_id=?1 AND (gf.hash IS NULL OR gf.crc32 IS NULL OR gf.md5 IS NULL OR gf.sha1 IS NULL OR gf.sha512 IS NULL OR ((gf.extension='zip' OR gf.extension='7z') AND gf.archive_hash IS NULL))", [gid.clone()], |r| r.get(0)).unwrap_or(0);
        if total > 0 && unhashed == 0 {
            eligible.push(gid.clone());
        } else {
            ignored += 1;
        }
    }
    let total_eligible = eligible.len();
    let total_games = game_ids.len();
    let _ = app.emit("enrich-progress", serde_json::json!({"total": total_games, "eligible": total_eligible, "ignored": ignored, "done": 0, "success": 0, "no_result": 0, "errors": 0, "status": "scanning", "game": null}));
    let mut done = 0usize;
    let mut success = 0usize;
    let mut no_result = 0usize;
    let mut errors = 0usize;
    for gid in &eligible {
        // get title
        let title: String = {
            let conn = state.db.lock().unwrap();
            conn.query_row("SELECT title FROM games WHERE id=?1", [gid.clone()], |r| {
                r.get(0)
            })
            .unwrap_or_else(|_| gid.clone())
        };
        let _ = app.emit("enrich-progress", serde_json::json!({"total": total_games, "eligible": total_eligible, "ignored": ignored, "done": done, "success": success, "no_result": no_result, "errors": errors, "status": "searching", "game": title}));
        match enrich_game_inner(&state, gid.clone(), "screenscraper".to_string()).await {
            Ok(v) => {
                let has_desc = v
                    .get("has_description")
                    .and_then(|x| x.as_bool())
                    .unwrap_or(false);
                if has_desc {
                    success += 1;
                } else {
                    no_result += 1;
                }
            }
            Err(_) => {
                errors += 1;
            }
        }
        done += 1;
        let _ = app.emit("enrich-progress", serde_json::json!({"total": total_games, "eligible": total_eligible, "ignored": ignored, "done": done, "success": success, "no_result": no_result, "errors": errors, "status": "searching", "game": title}));
    }
    let _ = app.emit("enrich-progress", serde_json::json!({"total": total_games, "eligible": total_eligible, "ignored": ignored, "done": done, "success": success, "no_result": no_result, "errors": errors, "status": "done", "game": null}));
    Ok(done)
}

#[tauri::command]
pub fn ss_no_result_game_ids(state: State<AppState>) -> Result<Vec<String>> {
    let conn = state.db.lock().unwrap();
    let mut stmt =
        conn.prepare("SELECT DISTINCT game_id FROM metadata_records WHERE field='ss_no_result'")?;
    let rows = stmt.query_map([], |r| r.get(0))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}
