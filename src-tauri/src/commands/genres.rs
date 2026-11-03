use crate::error::Result;
use crate::error::RetroError;
use crate::state::AppState;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn list_genres(state: State<'_, AppState>) -> Result<Vec<String>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT name FROM genres WHERE parent_id = '0' OR parent_id IS NULL ORDER BY name COLLATE NOCASE")?;
    let rows = stmt.query_map([], |r| r.get(0))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub async fn sync_genres_inner(state: &AppState) -> Result<usize> {
    let cfg = crate::config::load();
    let url = format!(
        "https://www.screenscraper.fr/api2/genresListe.php?devid={}&devpassword={}&softname=RETROVERSE&output=json&ssid={}&sspassword={}",
        crate::metadata::urlencoding(&cfg.screenscraper_devid),
        crate::metadata::urlencoding(&cfg.screenscraper_devpassword),
        crate::metadata::urlencoding(&cfg.screenscraper_ssid),
        crate::metadata::urlencoding(&cfg.screenscraper_sspassword),
    );
    let resp = ureq::get(&url)
        .call()
        .map_err(|e| RetroError::Other(e.to_string()))?;
    let mut r = resp;
    let json: serde_json::Value = r
        .body_mut()
        .read_json()
        .map_err(|e| RetroError::Other(e.to_string()))?;
    let conn = state.db.lock().unwrap();
    let mut count = 0usize;
    if let Some(arr) = json["response"]["genres"].as_array() {
        for g in arr {
            let id = g["id"]
                .as_str()
                .or_else(|| g["id"].as_i64().map(|_| ""))
                .unwrap_or("");
            // id may be number; handle via as_i64
            let id = if id.is_empty() {
                g["id"].as_i64().map(|n| n.to_string()).unwrap_or_default()
            } else {
                id.to_string()
            };
            let name = g["nom_fr"]
                .as_str()
                .or_else(|| g["nom_en"].as_str())
                .unwrap_or("")
                .to_string();
            if name.is_empty() {
                continue;
            }
            let parent = g["parent"]
                .as_str()
                .or_else(|| g["parent"].as_i64().map(|_| ""))
                .map(|s| s.to_string());
            let parent = parent
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "0".to_string());
            conn.execute(
                "INSERT OR REPLACE INTO genres (id, name, parent_id) VALUES (?1, ?2, ?3)",
                params![id, name, parent],
            )?;
            count += 1;
        }
    }
    Ok(count)
}

#[tauri::command]
pub async fn sync_genres(state: State<'_, AppState>) -> Result<usize> {
    sync_genres_inner(&state).await
}
