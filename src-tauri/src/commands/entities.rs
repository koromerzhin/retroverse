use crate::error::Result;
use crate::models::{Developer, Genre, Publisher};
use crate::state::AppState;
use chrono::Utc;
use rusqlite::params;
use tauri::State;

#[derive(serde::Deserialize)]
pub struct GenrePayload {
    pub name_en: Option<String>,
    pub name_fr: Option<String>,
    pub name_de: Option<String>,
    pub name_es: Option<String>,
    pub name_it: Option<String>,
    pub name_pt: Option<String>,
    pub parent_id: Option<String>,
    pub ss_id: Option<String>,
}
use uuid::Uuid;

#[derive(serde::Serialize)]
pub struct SyncStats {
    pub already: usize,
    pub added: usize,
    pub games_analyzed: usize,
}

#[tauri::command]
pub fn list_developers_full(state: State<'_, AppState>) -> Result<Vec<Developer>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id,name,ss_id,created_at,updated_at FROM developers ORDER BY name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([], |r| {
        let created: String = r.get(3)?;
        let updated: String = r.get(4)?;
        Ok(Developer {
            id: r.get(0)?,
            name: r.get(1)?,
            ss_id: r.get(2)?,
            created_at: created.parse().unwrap_or_else(|_| Utc::now()),
            updated_at: updated.parse().unwrap_or_else(|_| Utc::now()),
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[tauri::command]
pub fn add_developer(
    state: State<'_, AppState>,
    name: String,
    ss_id: Option<String>,
) -> Result<Developer> {
    let conn = state.db.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO developers (id,name,ss_id,created_at,updated_at) VALUES (?1,?2,?3,?4,?5)",
        params![id, name, ss_id, now, now],
    )?;
    Ok(Developer {
        id,
        name,
        ss_id,
        created_at: now.parse().unwrap(),
        updated_at: now.parse().unwrap(),
    })
}

#[tauri::command]
pub fn update_developer(
    state: State<'_, AppState>,
    id: String,
    name: String,
    ss_id: Option<String>,
) -> Result<()> {
    let conn = state.db.lock().unwrap();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE developers SET name=?1, ss_id=?2, updated_at=?3 WHERE id=?4",
        params![name, ss_id, now, id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn delete_developer(state: State<'_, AppState>, id: String) -> Result<()> {
    let conn = state.db.lock().unwrap();
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM game_developers WHERE developer_id=?1",
        [id.clone()],
        |r| r.get(0),
    )?;
    if count > 0 {
        return Err(crate::error::RetroError::Invalid(
            "Developer is used by games".into(),
        ));
    }
    conn.execute("DELETE FROM developers WHERE id=?1", [id])?;
    Ok(())
}

#[tauri::command]
pub fn list_publishers_full(state: State<'_, AppState>) -> Result<Vec<Publisher>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id,name,ss_id,created_at,updated_at FROM publishers ORDER BY name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([], |r| {
        let created: String = r.get(3)?;
        let updated: String = r.get(4)?;
        Ok(Publisher {
            id: r.get(0)?,
            name: r.get(1)?,
            ss_id: r.get(2)?,
            created_at: created.parse().unwrap_or_else(|_| Utc::now()),
            updated_at: updated.parse().unwrap_or_else(|_| Utc::now()),
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[tauri::command]
pub fn add_publisher(
    state: State<'_, AppState>,
    name: String,
    ss_id: Option<String>,
) -> Result<Publisher> {
    let conn = state.db.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO publishers (id,name,ss_id,created_at,updated_at) VALUES (?1,?2,?3,?4,?5)",
        params![id, name, ss_id, now, now],
    )?;
    Ok(Publisher {
        id,
        name,
        ss_id,
        created_at: now.parse().unwrap(),
        updated_at: now.parse().unwrap(),
    })
}

#[tauri::command]
pub fn update_publisher(
    state: State<'_, AppState>,
    id: String,
    name: String,
    ss_id: Option<String>,
) -> Result<()> {
    let conn = state.db.lock().unwrap();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE publishers SET name=?1, ss_id=?2, updated_at=?3 WHERE id=?4",
        params![name, ss_id, now, id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn delete_publisher(state: State<'_, AppState>, id: String) -> Result<()> {
    let conn = state.db.lock().unwrap();
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM game_publishers WHERE publisher_id=?1",
        [id.clone()],
        |r| r.get(0),
    )?;
    if count > 0 {
        return Err(crate::error::RetroError::Invalid(
            "Publisher is used by games".into(),
        ));
    }
    conn.execute("DELETE FROM publishers WHERE id=?1", [id])?;
    Ok(())
}

#[tauri::command]
pub fn list_genres_full(state: State<'_, AppState>) -> Result<Vec<Genre>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id,name_en,name_fr,name_de,name_es,name_it,name_pt,parent_id,ss_id,created_at,updated_at FROM genres ORDER BY COALESCE(name_fr,name_en) COLLATE NOCASE")?;
    let rows = stmt.query_map([], |r| {
        let created: String = r.get(9)?;
        let updated: String = r.get(10)?;
        Ok(Genre {
            id: r.get(0)?,
            name_en: r.get(1)?,
            name_fr: r.get(2)?,
            name_de: r.get(3)?,
            name_es: r.get(4)?,
            name_it: r.get(5)?,
            name_pt: r.get(6)?,
            parent_id: r.get(7)?,
            ss_id: r.get(8)?,
            created_at: created.parse().unwrap_or_else(|_| Utc::now()),
            updated_at: updated.parse().unwrap_or_else(|_| Utc::now()),
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[tauri::command]
pub fn add_genre(state: State<'_, AppState>, payload: GenrePayload) -> Result<Genre> {
    let conn = state.db.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO genres (id,name_en,name_fr,name_de,name_es,name_it,name_pt,parent_id,ss_id,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![id, payload.name_en, payload.name_fr, payload.name_de, payload.name_es, payload.name_it, payload.name_pt, payload.parent_id, payload.ss_id, now, now],
    )?;
    Ok(Genre {
        id,
        name_en: payload.name_en,
        name_fr: payload.name_fr,
        name_de: payload.name_de,
        name_es: payload.name_es,
        name_it: payload.name_it,
        name_pt: payload.name_pt,
        parent_id: payload.parent_id,
        ss_id: payload.ss_id,
        created_at: now.parse().unwrap(),
        updated_at: now.parse().unwrap(),
    })
}

#[tauri::command]
pub fn update_genre(state: State<'_, AppState>, id: String, payload: GenrePayload) -> Result<()> {
    let conn = state.db.lock().unwrap();
    let now = Utc::now().to_rfc3339();
    if payload.parent_id.as_deref() == Some(id.as_str()) {
        return Err(crate::error::RetroError::Invalid("Circular parent".into()));
    }
    conn.execute(
        "UPDATE genres SET name_en=?1,name_fr=?2,name_de=?3,name_es=?4,name_it=?5,name_pt=?6,parent_id=?7,ss_id=?8,updated_at=?9 WHERE id=?10",
        params![payload.name_en, payload.name_fr, payload.name_de, payload.name_es, payload.name_it, payload.name_pt, payload.parent_id, payload.ss_id, now, id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn delete_genre(state: State<'_, AppState>, id: String) -> Result<()> {
    let conn = state.db.lock().unwrap();
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM game_genres WHERE genre_id=?1",
        [id.clone()],
        |r| r.get(0),
    )?;
    if count > 0 {
        return Err(crate::error::RetroError::Invalid(
            "Genre is used by games".into(),
        ));
    }
    let child: i64 = conn.query_row(
        "SELECT COUNT(*) FROM genres WHERE parent_id=?1",
        [id.clone()],
        |r| r.get(0),
    )?;
    if child > 0 {
        return Err(crate::error::RetroError::Invalid(
            "Genre has subgenres".into(),
        ));
    }
    conn.execute("DELETE FROM genres WHERE id=?1", [id])?;
    Ok(())
}

fn json_str(v: &serde_json::Value, path: &[&str]) -> Option<String> {
    let mut cur = v;
    for k in path {
        cur = cur.get(*k)?;
    }
    cur.as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn sync_publishers_inner(state: &AppState) -> Result<SyncStats> {
    let conn = state.db.lock().unwrap();
    let now = chrono::Utc::now().to_rfc3339();
    // Collect game -> publisher name
    let mut game_pubs: Vec<(String, String)> = Vec::new();
    let mut stmt = conn.prepare("SELECT id, metadata_json FROM games")?;
    let games: Vec<(String, Option<String>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();
    let total_games = games.len();
    for (gid, mj) in &games {
        let mut name: Option<String> = None;
        if let Some(j) = mj
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(j)
        {
            name = json_str(&v, &["editeur", "text"])
                .or_else(|| json_str(&v, &["publisher"]))
                .or_else(|| json_str(&v, &["editeur"]));
        }
        if name.is_none() {
            name = conn
                .query_row(
                    "SELECT value FROM metadata_records WHERE game_id=?1 AND field='publisher' LIMIT 1",
                    [gid.clone()],
                    |r| r.get::<_, String>(0),
                )
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
        }
        if let Some(n) = name {
            game_pubs.push((gid.clone(), n));
        }
    }
    // Distinct names
    let distinct: std::collections::BTreeSet<String> =
        game_pubs.iter().map(|(_, n)| n.clone()).collect();
    let mut already = 0usize;
    let mut added = 0usize;
    let mut name_to_id: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for name in &distinct {
        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM publishers WHERE name=?1 COLLATE NOCASE",
                [name.clone()],
                |r| r.get(0),
            )
            .ok();
        if let Some(id) = existing {
            already += 1;
            name_to_id.insert(name.clone(), id);
        } else {
            let id = Uuid::new_v4().to_string();
            let _ = conn.execute(
                "INSERT INTO publishers (id,name,ss_id,created_at,updated_at) VALUES (?1,?2,NULL,?3,?4)",
                params![id, name, now, now],
            );
            added += 1;
            name_to_id.insert(name.clone(), id);
        }
    }
    // Link games
    for (gid, name) in &game_pubs {
        if let Some(pid) = name_to_id.get(name) {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM game_publishers WHERE game_id=?1 AND publisher_id=?2",
                    params![gid, pid],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap_or(0)
                > 0;
            if !exists {
                let _ = conn.execute(
                    "INSERT OR IGNORE INTO game_publishers (game_id, publisher_id) VALUES (?1,?2)",
                    params![gid, pid],
                );
            }
        }
    }
    Ok(SyncStats {
        already,
        added,
        games_analyzed: total_games,
    })
}

pub fn sync_developers_inner(state: &AppState) -> Result<SyncStats> {
    let conn = state.db.lock().unwrap();
    let now = chrono::Utc::now().to_rfc3339();
    let mut game_devs: Vec<(String, String)> = Vec::new();
    let mut stmt = conn.prepare("SELECT id, metadata_json FROM games")?;
    let games: Vec<(String, Option<String>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();
    let total_games = games.len();
    for (gid, mj) in &games {
        let mut name: Option<String> = None;
        if let Some(j) = mj
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(j)
        {
            name = json_str(&v, &["developpeur", "text"])
                .or_else(|| json_str(&v, &["developer"]))
                .or_else(|| json_str(&v, &["developpeur"]));
        }
        if name.is_none() {
            name = conn
                .query_row(
                    "SELECT value FROM metadata_records WHERE game_id=?1 AND field='developer' LIMIT 1",
                    [gid.clone()],
                    |r| r.get::<_, String>(0),
                )
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
        }
        if let Some(n) = name {
            game_devs.push((gid.clone(), n));
        }
    }
    let distinct: std::collections::BTreeSet<String> =
        game_devs.iter().map(|(_, n)| n.clone()).collect();
    let mut already = 0usize;
    let mut added = 0usize;
    let mut name_to_id: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for name in &distinct {
        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM developers WHERE name=?1 COLLATE NOCASE",
                [name.clone()],
                |r| r.get(0),
            )
            .ok();
        if let Some(id) = existing {
            already += 1;
            name_to_id.insert(name.clone(), id);
        } else {
            let id = Uuid::new_v4().to_string();
            let _ = conn.execute(
                "INSERT INTO developers (id,name,ss_id,created_at,updated_at) VALUES (?1,?2,NULL,?3,?4)",
                params![id, name, now, now],
            );
            added += 1;
            name_to_id.insert(name.clone(), id);
        }
    }
    for (gid, name) in &game_devs {
        if let Some(did) = name_to_id.get(name) {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM game_developers WHERE game_id=?1 AND developer_id=?2",
                    params![gid, did],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap_or(0)
                > 0;
            if !exists {
                let _ = conn.execute(
                    "INSERT OR IGNORE INTO game_developers (game_id, developer_id) VALUES (?1,?2)",
                    params![gid, did],
                );
            }
        }
    }
    Ok(SyncStats {
        already,
        added,
        games_analyzed: total_games,
    })
}

pub fn sync_entities_from_metadata_inner(state: &AppState) -> Result<usize> {
    let a = sync_publishers_inner(state)?.added;
    let b = sync_developers_inner(state)?.added;
    Ok(a + b)
}

#[tauri::command]
pub fn sync_entities_from_metadata(state: State<'_, AppState>) -> Result<usize> {
    sync_entities_from_metadata_inner(&state)
}

#[tauri::command]
pub fn sync_publishers(state: State<'_, AppState>) -> Result<SyncStats> {
    sync_publishers_inner(&state)
}

#[tauri::command]
pub fn sync_developers(state: State<'_, AppState>) -> Result<SyncStats> {
    sync_developers_inner(&state)
}
