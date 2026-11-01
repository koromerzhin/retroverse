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

    let prov = metadata::provider_by_name_with_config(&provider, &crate::config::load())
        .ok_or_else(|| RetroError::Invalid("unknown provider".into()))?;

    let res = prov.fetch(&title, platform.as_deref())?;

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
            let exists: bool = conn.query_row(
                "SELECT COUNT(*) FROM metadata_records WHERE game_id=?1 AND field='developer'",
                [game_id.clone()],
                |r| r.get::<_, i64>(0),
            ).unwrap_or(0) > 0;
            if !exists {
                let id = crate::models::new_id();
                conn.execute(
                    "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![id, game_id, provider, "developer", dev, provider, now],
                )?;
            }
        }
        if let Some(cov) = &res.cover_url {
            let exists: bool = conn.query_row(
                "SELECT COUNT(*) FROM media_assets WHERE game_id=?1 AND kind='cover'",
                [game_id.clone()],
                |r| r.get::<_, i64>(0),
            ).unwrap_or(0) > 0;
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
            let exists: bool = conn.query_row(
                "SELECT COUNT(*) FROM metadata_records WHERE game_id=?1 AND field='alt_title'",
                [game_id.clone()],
                |r| r.get::<_, i64>(0),
            ).unwrap_or(0) > 0;
            if !exists {
                // don't overwrite title automatically; store as alternative
                let id = crate::models::new_id();
                conn.execute(
                    "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![id, game_id, provider, "alt_title", t, provider, now],
                )?;
            }
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
    let mut mstmt = conn.prepare("SELECT kind,url FROM media_assets WHERE game_id=?1 ORDER BY cached_at DESC")?;
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
