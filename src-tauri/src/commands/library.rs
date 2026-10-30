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

#[tauri::command]
pub fn list_games(
    state: State<AppState>,
    platform: Option<String>,
    search: Option<String>,
) -> Result<Vec<LibraryEntry>> {
    let conn = state.db.lock().unwrap();
    let platforms = crate::identify::load_platforms(&conn)?;
    let pmap: std::collections::HashMap<String, Platform> =
        platforms.into_iter().map(|p| (p.id.clone(), p)).collect();

    // Build query
    let mut sql =
        String::from("SELECT g.id,g.title,g.platform_id,g.created_at,g.updated_at FROM games g");
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
    if !where_clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&where_clauses.join(" AND "));
    }
    sql.push_str(" ORDER BY g.title COLLATE NOCASE ASC");

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
        // files
        let mut fstmt = conn.prepare(
            "SELECT gf.id,gf.source_id,gf.path,gf.filename,gf.extension,gf.size,gf.modified_at,gf.hash,gf.platform_id,gf.created_at \
             FROM game_files gf JOIN game_file_links l ON l.file_id=gf.id WHERE l.game_id=?1",
        )?;
        let frows = fstmt.query_map([game.id.clone()], |r| {
            let created: String = r.get(9)?;
            Ok(crate::models::GameFile {
                id: r.get(0)?,
                source_id: r.get(1)?,
                path: r.get(2)?,
                filename: r.get(3)?,
                extension: r.get(4)?,
                size: r.get(5)?,
                modified_at: r.get(6)?,
                hash: r.get(7)?,
                platform_id: r.get(8)?,
                created_at: created.parse().unwrap_or_else(|_| chrono::Utc::now()),
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
    Ok(entries)
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
        "SELECT gf.id,gf.source_id,gf.path,gf.filename,gf.extension,gf.size,gf.modified_at,gf.hash,gf.platform_id,gf.created_at \
         FROM game_files gf JOIN game_file_links l ON l.file_id=gf.id WHERE l.game_id=?1",
    )?;
    let frows = fstmt.query_map([game.id.clone()], |r| {
        let created: String = r.get(9)?;
        Ok(crate::models::GameFile {
            id: r.get(0)?,
            source_id: r.get(1)?,
            path: r.get(2)?,
            filename: r.get(3)?,
            extension: r.get(4)?,
            size: r.get(5)?,
            modified_at: r.get(6)?,
            hash: r.get(7)?,
            platform_id: r.get(8)?,
            created_at: created.parse().unwrap_or_else(|_| chrono::Utc::now()),
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

// unused Row import guard
#[allow(dead_code)]
fn _row(_r: &Row) {}
