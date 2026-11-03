use crate::error::Result;
use crate::models::{Game, Platform};
use crate::state::AppState;
use rusqlite::{Row, params};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct LibraryEntry {
    pub game: Game,
    pub platform: Option<Platform>,
    pub files: Vec<crate::models::GameFile>,
    pub file_count: usize,
}

#[derive(serde::Serialize)]
pub struct PagedGames {
    pub items: Vec<LibraryEntry>,
    pub total: usize,
}

#[tauri::command]
pub fn list_games(
    state: State<AppState>,
    platform: Option<String>,
    search: Option<String>,
    developer: Option<String>,
    publisher: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<PagedGames> {
    let conn = state.db.lock().unwrap();
    let platforms = crate::identify::load_platforms(&conn)?;
    let pmap: std::collections::HashMap<String, Platform> =
        platforms.into_iter().map(|p| (p.id.clone(), p)).collect();

    let limit = limit.unwrap_or(48).min(500);
    let offset = offset.unwrap_or(0);

    let mut where_clauses: Vec<String> = Vec::new();
    if let Some(plat) = &platform {
        where_clauses.push(format!("g.platform_id = '{}'", plat.replace('\'', "''")));
    }
    if let Some(q) = &search
        && !q.trim().is_empty()
    {
        where_clauses.push(format!(
            "LOWER(g.title) LIKE '%{}%'",
            q.to_lowercase().replace('\'', "''").replace('%', "\\%")
        ));
    }
    if let Some(dev) = &developer {
        where_clauses.push(format!(
            "EXISTS (SELECT 1 FROM metadata_records m WHERE m.game_id = g.id AND m.field = 'developer' AND m.value = '{}')",
            dev.replace('\'', "''")
        ));
    }
    if let Some(publ) = &publisher {
        where_clauses.push(format!(
            "EXISTS (SELECT 1 FROM metadata_records m WHERE m.game_id = g.id AND m.field = 'publisher' AND m.value = '{}')",
            publ.replace('\'', "''")
        ));
    }
    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_clauses.join(" AND "))
    };

    let count_sql = format!("SELECT COUNT(*) FROM games g{}", where_sql);
    let total: i64 = conn.query_row(&count_sql, [], |r| r.get(0))?;
    let total = total as usize;

    let sql = format!(
        "SELECT g.id,g.title,g.platform_id,g.created_at,g.updated_at FROM games g{} ORDER BY g.title COLLATE NOCASE ASC LIMIT {} OFFSET {}",
        where_sql, limit, offset
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |r| {
        let created: String = r.get(3)?;
        let updated: String = r.get(4)?;
        Ok(Game {
            id: r.get(0)?,
            title: r.get(1)?,
            platform_id: r.get(2)?,
            created_at: created.parse().unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: updated.parse().unwrap_or_else(|_| chrono::Utc::now()),
        })
    })?;

    let mut entries = Vec::new();
    for gr in rows {
        let game = gr?;
        let mut fstmt = conn.prepare(
            "SELECT gf.id,gf.source_id,gf.path,gf.filename,gf.extension,gf.size,gf.modified_at,gf.hash,gf.crc32,gf.md5,gf.sha1,gf.sha512,gf.zip_inner,gf.archive_hash,gf.archive_crc32,gf.archive_md5,gf.archive_sha1,gf.archive_sha512,gf.platform_id,gf.created_at,gf.is_dir \
             FROM game_files gf JOIN game_file_links l ON l.file_id=gf.id WHERE l.game_id=?1",
        )?;
        let frows = fstmt.query_map([game.id.clone()], |r| {
            let created: String = r.get(19)?;
            Ok(crate::models::GameFile {
                id: r.get(0)?,
                source_id: r.get(1)?,
                path: r.get(2)?,
                filename: r.get(3)?,
                extension: r.get(4)?,
                size: r.get(5)?,
                modified_at: r.get(6)?,
                hash: r.get(7)?,
                crc32: r.get(8)?,
                md5: r.get(9)?,
                sha1: r.get(10)?,
                sha512: r.get(11)?,
                zip_inner: r.get(12)?,
                archive_hash: r.get(13)?,
                archive_crc32: r.get(14)?,
                archive_md5: r.get(15)?,
                archive_sha1: r.get(16)?,
                archive_sha512: r.get(17)?,
                platform_id: r.get(18)?,
                created_at: created.parse().unwrap_or_else(|_| chrono::Utc::now()),
                is_dir: r.get::<_, i64>(20).unwrap_or(0) != 0,
            })
        })?;
        let mut files = Vec::new();
        for fr in frows {
            files.push(fr?);
        }
        let platform = game
            .platform_id
            .as_ref()
            .and_then(|pid| pmap.get(pid).cloned());
        entries.push(LibraryEntry {
            game,
            platform,
            file_count: files.len(),
            files,
        });
    }
    Ok(PagedGames {
        items: entries,
        total,
    })
}

#[tauri::command]
pub fn get_game(state: State<AppState>, id: String) -> Result<LibraryEntry> {
    let conn = state.db.lock().unwrap();
    let platforms = crate::identify::load_platforms(&conn)?;
    let pmap: std::collections::HashMap<String, Platform> =
        platforms.into_iter().map(|p| (p.id.clone(), p)).collect();

    let game: Game = conn.query_row(
        "SELECT id,title,platform_id,created_at,updated_at FROM games WHERE id=?1",
        [id.clone()],
        |r| {
            let created: String = r.get(3)?;
            let updated: String = r.get(4)?;
            Ok(Game {
                id: r.get(0)?,
                title: r.get(1)?,
                platform_id: r.get(2)?,
                created_at: created.parse().unwrap_or_else(|_| chrono::Utc::now()),
                updated_at: updated.parse().unwrap_or_else(|_| chrono::Utc::now()),
            })
        },
    )?;

    let mut fstmt = conn.prepare(
        "SELECT gf.id,gf.source_id,gf.path,gf.filename,gf.extension,gf.size,gf.modified_at,gf.hash,gf.crc32,gf.md5,gf.sha1,gf.sha512,gf.zip_inner,gf.archive_hash,gf.archive_crc32,gf.archive_md5,gf.archive_sha1,gf.archive_sha512,gf.platform_id,gf.created_at,gf.is_dir \
         FROM game_files gf JOIN game_file_links l ON l.file_id=gf.id WHERE l.game_id=?1",
    )?;
    let frows = fstmt.query_map([game.id.clone()], |r| {
        let created: String = r.get(19)?;
        Ok(crate::models::GameFile {
            id: r.get(0)?,
            source_id: r.get(1)?,
            path: r.get(2)?,
            filename: r.get(3)?,
            extension: r.get(4)?,
            size: r.get(5)?,
            modified_at: r.get(6)?,
            hash: r.get(7)?,
            crc32: r.get(8)?,
            md5: r.get(9)?,
            sha1: r.get(10)?,
            sha512: r.get(11)?,
            zip_inner: r.get(12)?,
            archive_hash: r.get(13)?,
            archive_crc32: r.get(14)?,
            archive_md5: r.get(15)?,
            archive_sha1: r.get(16)?,
            archive_sha512: r.get(17)?,
            platform_id: r.get(18)?,
            created_at: created.parse().unwrap_or_else(|_| chrono::Utc::now()),
            is_dir: r.get::<_, i64>(20).unwrap_or(0) != 0,
        })
    })?;
    let mut files = Vec::new();
    for fr in frows {
        files.push(fr?);
    }
    let platform = game
        .platform_id
        .as_ref()
        .and_then(|pid| pmap.get(pid).cloned());
    Ok(LibraryEntry {
        game,
        platform,
        file_count: files.len(),
        files,
    })
}

#[tauri::command]
pub fn correct_game_title(state: State<AppState>, game_id: String, title: String) -> Result<()> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "UPDATE games SET title=?1, updated_at=?2 WHERE id=?3",
        params![title, chrono::Utc::now().to_rfc3339(), game_id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn link_file_to_game(state: State<AppState>, file_id: String, game_id: String) -> Result<()> {
    let conn = state.db.lock().unwrap();
    // record manual correction
    let cid = crate::models::new_id();
    conn.execute(
        "INSERT INTO user_corrections (id,file_id,game_id,created_at) VALUES (?1,?2,?3,?4)",
        params![cid, file_id, game_id, chrono::Utc::now().to_rfc3339()],
    )?;
    conn.execute(
        "DELETE FROM game_file_links WHERE file_id=?1",
        [file_id.clone()],
    )?;
    conn.execute(
        "INSERT INTO game_file_links (file_id,game_id,confidence,manual) VALUES (?1,?2,1.0,1)",
        params![file_id, game_id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn platform_file_counts(state: State<AppState>) -> Result<serde_json::Value> {
    let conn = state.db.lock().unwrap();
    let mut stmt =
        conn.prepare("SELECT platform_id, COUNT(*) FROM game_files GROUP BY platform_id")?;
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, Option<String>>(0)?, r.get::<_, i64>(1)?))
    })?;
    let mut map = serde_json::Map::new();
    for r in rows {
        let (pid, c) = r?;
        if let Some(p) = pid {
            map.insert(p, serde_json::json!(c));
        }
    }
    Ok(serde_json::Value::Object(map))
}

#[tauri::command]
pub fn platform_game_counts(state: State<AppState>) -> Result<serde_json::Value> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT platform_id, COUNT(*) FROM games GROUP BY platform_id")?;
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, Option<String>>(0)?, r.get::<_, i64>(1)?))
    })?;
    let mut map = serde_json::Map::new();
    for r in rows {
        let (pid, c) = r?;
        if let Some(p) = pid {
            map.insert(p, serde_json::json!(c));
        }
    }
    Ok(serde_json::Value::Object(map))
}

#[tauri::command]
pub fn stats(state: State<AppState>) -> Result<serde_json::Value> {
    let conn = state.db.lock().unwrap();
    let games: i64 = conn.query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))?;
    let files: i64 = conn.query_row("SELECT COUNT(*) FROM game_files", [], |r| r.get(0))?;
    let sources: i64 = conn.query_row("SELECT COUNT(*) FROM game_sources", [], |r| r.get(0))?;
    let platforms: i64 = conn.query_row("SELECT COUNT(*) FROM platforms", [], |r| r.get(0))?;
    Ok(serde_json::json!({
        "games": games,
        "files": files,
        "sources": sources,
        "platforms": platforms,
    }))
}

#[tauri::command]
pub fn search_games_fts(
    state: State<AppState>,
    query: String,
    platform: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<PagedGames> {
    let conn = state.db.lock().unwrap();
    let platforms = crate::identify::load_platforms(&conn)?;
    let pmap: std::collections::HashMap<String, Platform> =
        platforms.into_iter().map(|p| (p.id.clone(), p)).collect();

    // Sanitize query for FTS5 (avoid syntax errors on special chars).
    let safe_query: String = query
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();
    let safe_query = safe_query.trim().to_string();

    // if query empty, fallback list
    let rows = if safe_query.is_empty() {
        conn.prepare("SELECT id,title,platform_id,created_at,updated_at FROM games ORDER BY title")?
            .query_map([], |r| {
                let created: String = r.get(3)?;
                let updated: String = r.get(4)?;
                Ok(Game {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    platform_id: r.get(2)?,
                    created_at: created.parse().unwrap_or_else(|_| chrono::Utc::now()),
                    updated_at: updated.parse().unwrap_or_else(|_| chrono::Utc::now()),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
    } else {
        let mut stmt = conn.prepare(
            "SELECT g.id,g.title,g.platform_id,g.created_at,g.updated_at FROM games g \
             JOIN games_fts f ON f.rowid = g.rowid \
             WHERE games_fts MATCH ?1 ORDER BY rank",
        )?;
        stmt.query_map([format!("{}*", safe_query)], |r| {
            let created: String = r.get(3)?;
            let updated: String = r.get(4)?;
            Ok(Game {
                id: r.get(0)?,
                title: r.get(1)?,
                platform_id: r.get(2)?,
                created_at: created.parse().unwrap_or_else(|_| chrono::Utc::now()),
                updated_at: updated.parse().unwrap_or_else(|_| chrono::Utc::now()),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
    };

    let rows: Vec<Game> = if let Some(pf) = &platform {
        rows.into_iter()
            .filter(|g| g.platform_id.as_deref() == Some(pf.as_str()))
            .collect()
    } else {
        rows
    };
    let total = rows.len();
    let limit = limit.unwrap_or(48).min(500);
    let offset = offset.unwrap_or(0);
    let page_rows: Vec<Game> = rows.into_iter().skip(offset).take(limit).collect();

    let mut entries = Vec::new();
    for game in page_rows {
        let mut fstmt = conn.prepare(
            "SELECT gf.id,gf.source_id,gf.path,gf.filename,gf.extension,gf.size,gf.modified_at,gf.hash,gf.crc32,gf.md5,gf.sha1,gf.sha512,gf.zip_inner,gf.archive_hash,gf.archive_crc32,gf.archive_md5,gf.archive_sha1,gf.archive_sha512,gf.platform_id,gf.created_at,gf.is_dir \
             FROM game_files gf JOIN game_file_links l ON l.file_id=gf.id WHERE l.game_id=?1",
        )?;
        let frows = fstmt.query_map([game.id.clone()], |r| {
            let created: String = r.get(19)?;
            Ok(crate::models::GameFile {
                id: r.get(0)?,
                source_id: r.get(1)?,
                path: r.get(2)?,
                filename: r.get(3)?,
                extension: r.get(4)?,
                size: r.get(5)?,
                modified_at: r.get(6)?,
                hash: r.get(7)?,
                crc32: r.get(8)?,
                md5: r.get(9)?,
                sha1: r.get(10)?,
                sha512: r.get(11)?,
                zip_inner: r.get(12)?,
                archive_hash: r.get(13)?,
                archive_crc32: r.get(14)?,
                archive_md5: r.get(15)?,
                archive_sha1: r.get(16)?,
                archive_sha512: r.get(17)?,
                platform_id: r.get(18)?,
                created_at: created.parse().unwrap_or_else(|_| chrono::Utc::now()),
                is_dir: r.get::<_, i64>(20).unwrap_or(0) != 0,
            })
        })?;
        let mut files = Vec::new();
        for fr in frows {
            files.push(fr?);
        }
        let platform = game
            .platform_id
            .as_ref()
            .and_then(|pid| pmap.get(pid).cloned());
        entries.push(LibraryEntry {
            game,
            platform,
            file_count: files.len(),
            files,
        });
    }
    Ok(PagedGames {
        items: entries,
        total,
    })
}

#[tauri::command]
pub fn find_duplicates(state: State<AppState>) -> Result<Vec<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    // groups by lower(title) + platform having more than one game
    let mut stmt = conn.prepare(
        "SELECT LOWER(title), platform_id, COUNT(*) as c FROM games GROUP BY LOWER(title), platform_id HAVING c > 1",
    )?;
    let groups = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
    })?;
    let mut out = Vec::new();
    for g in groups {
        let (t, pid) = g?;
        let mut gstmt = conn.prepare("SELECT id,title FROM games WHERE LOWER(title)=?1 AND (platform_id IS ?2 OR platform_id=?2)")?;
        let games: Vec<(String, String)> = gstmt
            .query_map(params![t, pid], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        out.push(serde_json::json!({"title": t, "platform_id": pid, "games": games}));
    }
    Ok(out)
}

#[tauri::command]
pub fn list_collections(state: State<AppState>) -> Result<Vec<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id,name,created_at FROM collections ORDER BY name")?;
    let rows = stmt.query_map([], |r| {
        Ok(serde_json::json!({"id": r.get::<_, String>(0)?, "name": r.get::<_, String>(1)?, "created_at": r.get::<_, String>(2)?}))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[tauri::command]
pub fn create_collection(state: State<AppState>, name: String) -> Result<serde_json::Value> {
    let conn = state.db.lock().unwrap();
    let id = crate::models::new_id();
    conn.execute(
        "INSERT INTO collections (id,name,created_at) VALUES (?1,?2,?3)",
        params![id, name, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(serde_json::json!({"id": id, "name": name}))
}

#[tauri::command]
pub fn add_game_to_collection(
    state: State<AppState>,
    collection_id: String,
    game_id: String,
) -> Result<()> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "INSERT OR IGNORE INTO collection_games (collection_id,game_id) VALUES (?1,?2)",
        params![collection_id, game_id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn remove_game_from_collection(
    state: State<AppState>,
    collection_id: String,
    game_id: String,
) -> Result<()> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "DELETE FROM collection_games WHERE collection_id=?1 AND game_id=?2",
        params![collection_id, game_id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn collection_games(state: State<AppState>, collection_id: String) -> Result<Vec<String>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT game_id FROM collection_games WHERE collection_id=?1")?;
    let rows = stmt.query_map([collection_id], |r| r.get(0))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

// unused Row import guard
#[allow(dead_code)]
fn _row(_r: &Row) {}
