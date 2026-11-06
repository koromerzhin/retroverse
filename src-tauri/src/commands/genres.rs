use crate::error::Result;
use crate::error::RetroError;
use crate::state::AppState;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn list_genres(state: State<'_, AppState>) -> Result<Vec<String>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT COALESCE(name_fr, name_en, name_de, name_es, name_it, name_pt) FROM genres WHERE parent_id = '0' OR parent_id IS NULL ORDER BY 1 COLLATE NOCASE")?;
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
    let now = chrono::Utc::now().to_rfc3339();
    let mut count = 0usize;
    if let Some(obj) = json["response"]["genres"].as_object() {
        for (key, g) in obj {
            let id = g["id"]
                .as_i64()
                .map(|n| n.to_string())
                .unwrap_or_else(|| key.clone());
            let name_en = g["nom_en"].as_str().map(|s| s.to_string());
            let name_fr = g["nom_fr"].as_str().map(|s| s.to_string());
            let name_de = g["nom_de"].as_str().map(|s| s.to_string());
            let name_es = g["nom_es"].as_str().map(|s| s.to_string());
            let name_it = g["nom_it"].as_str().map(|s| s.to_string());
            let name_pt = g["nom_pt"].as_str().map(|s| s.to_string());
            let name = name_fr
                .clone()
                .or_else(|| name_en.clone())
                .unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let parent = g["parent"]
                .as_i64()
                .map(|n| n.to_string())
                .or_else(|| g["parent"].as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "0".to_string());
            let parent = if parent == "0" { None } else { Some(parent) };
            conn.execute(
                "INSERT OR REPLACE INTO genres (id, name_en, name_fr, name_de, name_es, name_it, name_pt, parent_id, ss_id, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![id, name_en, name_fr, name_de, name_es, name_it, name_pt, parent, Some(id.clone()), now, now],
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
