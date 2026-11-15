use crate::config;
use crate::error::{Result, RetroError};
use crate::state::AppState;
use rusqlite::params;
use serde::Serialize;
use tauri::State;

#[derive(Serialize, Clone)]
pub struct SgdbGame {
    pub id: i64,
    pub name: String,
}

#[tauri::command]
pub fn sgdb_search_game(title: String) -> Result<Vec<SgdbGame>> {
    let cfg = config::load();
    if cfg.sgdb_api_key.trim().is_empty() {
        return Err(RetroError::Invalid(
            "SteamGridDB API key not configured".into(),
        ));
    }
    let url = format!(
        "https://www.steamgriddb.com/api/v2/search/autocomplete/{}",
        urlencoding(&title)
    );
    let resp = ureq::get(&url)
        .header("Authorization", &format!("Bearer {}", cfg.sgdb_api_key))
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
    if let Some(data) = v.get("data").and_then(|x| x.as_array()) {
        for item in data {
            let id = item.get("id").and_then(|x| x.as_i64()).unwrap_or(0);
            let name = item
                .get("name")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            if id > 0 {
                out.push(SgdbGame { id, name });
            }
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn sgdb_fetch_artworks(
    state: State<AppState>,
    game_id: String,
    sgdb_game_id: i64,
) -> Result<usize> {
    let cfg = config::load();
    if cfg.sgdb_api_key.trim().is_empty() {
        return Err(RetroError::Invalid(
            "SteamGridDB API key not configured".into(),
        ));
    }
    let auth = format!("Bearer {}", cfg.sgdb_api_key);
    let base = "https://www.steamgriddb.com/api/v2";
    let kinds = [
        ("grids", "cover"),
        ("heroes", "hero"),
        ("logos", "logo"),
        ("icons", "icon"),
    ];
    let now = chrono::Utc::now().to_rfc3339();
    let cache_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("retroverse")
        .join("cache")
        .join("sgdb");
    let _ = std::fs::create_dir_all(&cache_dir);

    let conn = state.db.lock().unwrap();
    let mut total = 0usize;
    for (endpoint, kind) in kinds {
        let url = format!("{}/{}/game/{}", base, endpoint, sgdb_game_id);
        let resp = match ureq::get(&url).header("Authorization", &auth).call() {
            Ok(r) => r,
            Err(_) => continue,
        };
        let mut body = String::new();
        use std::io::Read;
        let _ = resp.into_body().as_reader().read_to_string(&mut body);
        let v: serde_json::Value = match serde_json::from_str(&body) {
            Ok(x) => x,
            Err(_) => continue,
        };
        let data = match v.get("data").and_then(|x| x.as_array()) {
            Some(d) => d,
            None => continue,
        };
        for item in data.iter().take(3) {
            let img_url = item
                .get("url")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            if img_url.is_empty() {
                continue;
            }
            // download
            let resp = match ureq::get(&img_url).call() {
                Ok(r) => r,
                Err(_) => continue,
            };
            let mut bytes = Vec::new();
            let _ = resp.into_body().as_reader().read_to_end(&mut bytes);
            if bytes.is_empty() {
                continue;
            }
            let ext = img_url.rsplit('.').next().unwrap_or("png");
            let ext = if ext.len() <= 4 { ext } else { "png" };
            let fname = format!("{}_{}_{}.{}", game_id, kind, crate::models::new_id(), ext);
            let dest = cache_dir.join(&fname);
            if std::fs::write(&dest, &bytes).is_err() {
                continue;
            }
            let id = crate::models::new_id();
            let _ = conn.execute(
                "INSERT INTO media_assets (id,game_id,kind,url,path,cached_at) VALUES (?1,?2,?3,?4,?5,?6)",
                params![id, game_id, kind, img_url, dest.to_string_lossy().to_string(), now],
            );
            total += 1;
        }
    }
    Ok(total)
}

fn urlencoding(s: &str) -> String {
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
