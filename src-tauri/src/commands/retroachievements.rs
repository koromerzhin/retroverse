//! Commands for retroachievements.

use crate::config;
use crate::error::{Result, RetroError};
use crate::state::AppState;
use rusqlite::params;
use serde::Serialize;
use tauri::State;

#[derive(Serialize, Clone)]
pub struct RaGame {
    pub id: i64,
    pub title: String,
    pub console: String,
}

#[derive(Serialize, Clone)]
pub struct RaAchievement {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub points: i64,
    pub badge: String,
    pub unlocked: bool,
}

#[tauri::command]
pub fn ra_search_game(title: String) -> Result<Vec<RaGame>> {
    let cfg = config::load();
    if cfg.ra_api_key.trim().is_empty() {
        return Err(RetroError::Invalid(
            "RetroAchievements API key not configured".into(),
        ));
    }
    let url = format!(
        "https://api.retroachievements.org/API/SearchGames.php?y={}&g={}",
        urlencoding(&cfg.ra_api_key),
        urlencoding(&title)
    );
    let resp = ureq::get(&url)
        .call()
        .map_err(|e| RetroError::Other(e.to_string()))?;
    let mut body = String::new();
    use std::io::Read;
    resp.into_body()
        .as_reader()
        .read_to_string(&mut body)
        .map_err(RetroError::Io)?;
    let v: serde_json::Value = serde_json::from_str(&body).map_err(RetroError::Serde)?;
    let mut out = Vec::new();
    if let Some(arr) = v.as_array() {
        for item in arr {
            let id = item.get("ID").and_then(|x| x.as_i64()).unwrap_or(0);
            let title = item
                .get("Title")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let console = item
                .get("ConsoleName")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            if id > 0 {
                out.push(RaGame { id, title, console });
            }
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn ra_fetch_achievements(
    state: State<AppState>,
    game_id: String,
    ra_game_id: i64,
) -> Result<usize> {
    let cfg = config::load();
    if cfg.ra_api_key.trim().is_empty() {
        return Err(RetroError::Invalid(
            "RetroAchievements API key not configured".into(),
        ));
    }
    // Get game achievements
    let url = format!(
        "https://api.retroachievements.org/API/GetGame.php?y={}&i={}",
        urlencoding(&cfg.ra_api_key),
        ra_game_id
    );
    let resp = ureq::get(&url)
        .call()
        .map_err(|e| RetroError::Other(e.to_string()))?;
    let mut body = String::new();
    use std::io::Read;
    resp.into_body()
        .as_reader()
        .read_to_string(&mut body)
        .map_err(RetroError::Io)?;
    let v: serde_json::Value = serde_json::from_str(&body).map_err(RetroError::Serde)?;

    // User progress (unlocked)
    let mut unlocked: std::collections::HashSet<i64> = std::collections::HashSet::new();
    if !cfg.ra_username.trim().is_empty() {
        let url2 = format!(
            "https://api.retroachievements.org/API/GetUserProgress.php?u={}&y={}&g={}",
            urlencoding(&cfg.ra_username),
            urlencoding(&cfg.ra_api_key),
            ra_game_id
        );
        if let Ok(resp2) = ureq::get(&url2).call() {
            let mut body2 = String::new();
            let _ = resp2.into_body().as_reader().read_to_string(&mut body2);
            if let Ok(v2) = serde_json::from_str::<serde_json::Value>(&body2)
                && let Some(ach) = v2.get("Achievements").and_then(|x| x.as_object())
            {
                for (k, val) in ach {
                    let has_date = val
                        .get("Date")
                        .and_then(|x| x.as_str())
                        .map(|s| !s.is_empty())
                        .unwrap_or(false);
                    if has_date && let Ok(id) = k.parse::<i64>() {
                        unlocked.insert(id);
                    }
                }
            }
        }
    }

    let now = chrono::Utc::now().to_rfc3339();
    let conn = state.db.lock().unwrap();
    let mut count = 0usize;
    if let Some(ach) = v.get("Achievements").and_then(|x| x.as_object()) {
        for (k, a) in ach {
            let ra_id: i64 = a
                .get("ID")
                .and_then(|x| x.as_i64())
                .or_else(|| k.parse().ok())
                .unwrap_or(0);
            let title = a
                .get("Title")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let desc = a
                .get("Description")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let points = a.get("Points").and_then(|x| x.as_i64()).unwrap_or(0);
            let badge = a
                .get("BadgeName")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let is_unlocked = unlocked.contains(&ra_id);
            let id = crate::models::new_id();
            conn.execute(
                "INSERT INTO game_achievements (id,game_id,ra_game_id,ra_achievement_id,title,description,points,badge_url,unlocked,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10) ON CONFLICT(game_id, ra_achievement_id) DO UPDATE SET title=excluded.title, description=excluded.description, points=excluded.points, badge_url=excluded.badge_url, unlocked=excluded.unlocked, fetched_at=excluded.fetched_at",
                params![id, game_id, ra_game_id, ra_id, title, desc, points, badge, if is_unlocked { 1 } else { 0 }, now],
            )?;
            count += 1;
        }
    }
    Ok(count)
}

#[tauri::command]
pub fn list_game_achievements(
    state: State<AppState>,
    game_id: String,
) -> Result<Vec<RaAchievement>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT ra_achievement_id, title, description, points, badge_url, unlocked FROM game_achievements WHERE game_id=?1 ORDER BY ra_achievement_id",
    )?;
    let rows = stmt.query_map([game_id], |r| {
        Ok(RaAchievement {
            id: r.get(0)?,
            title: r.get(1)?,
            description: r.get(2)?,
            points: r.get(3)?,
            badge: r.get(4)?,
            unlocked: r.get::<_, i64>(5)? != 0,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn urlencoding(s: &str) -> String {
    // Minimal URL encoding for query values.
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
