use crate::error::{Result, RetroError};
use crate::models::Classification;
use crate::state::AppState;
use chrono::Utc;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn list_classifications_full(state: State<'_, AppState>) -> Result<Vec<Classification>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT c.id, c.system_id, s.name, c.value, c.ss_id, c.description, c.created_at, c.updated_at FROM classifications c LEFT JOIN classification_systems s ON s.id = c.system_id ORDER BY s.name COLLATE NOCASE, c.value COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([], |r| {
        let created: String = r.get(6)?;
        let updated: String = r.get(7)?;
        Ok(Classification {
            id: r.get(0)?,
            system_id: r.get(1)?,
            system_name: r.get(2)?,
            value: r.get(3)?,
            ss_id: r.get(4)?,
            description: r.get(5)?,
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
pub fn list_game_classifications(
    state: State<'_, AppState>,
    game_id: String,
) -> Result<Vec<Classification>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT c.id, c.system_id, s.name, c.value, c.ss_id, c.description, c.created_at, c.updated_at FROM classifications c JOIN game_classifications gc ON gc.classification_id = c.id LEFT JOIN classification_systems s ON s.id = c.system_id WHERE gc.game_id=?1 ORDER BY s.name COLLATE NOCASE, c.value COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([game_id], |r| {
        let created: String = r.get(6)?;
        let updated: String = r.get(7)?;
        Ok(Classification {
            id: r.get(0)?,
            system_id: r.get(1)?,
            system_name: r.get(2)?,
            value: r.get(3)?,
            ss_id: r.get(4)?,
            description: r.get(5)?,
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
pub async fn sync_classifications(state: State<'_, AppState>) -> Result<usize> {
    let cfg = crate::config::load();
    let url = format!(
        "https://api.screenscraper.fr/api2/classificationsListe.php?devid={}&devpassword={}&softname=RETROVERSE&output=xml&ssid={}&sspassword={}",
        crate::metadata::urlencoding(&cfg.screenscraper_devid),
        crate::metadata::urlencoding(&cfg.screenscraper_devpassword),
        crate::metadata::urlencoding(&cfg.screenscraper_ssid),
        crate::metadata::urlencoding(&cfg.screenscraper_sspassword),
    );
    let resp = ureq::get(&url)
        .call()
        .map_err(|e| RetroError::Other(e.to_string()))?;
    let mut r = resp;
    let body = r
        .body_mut()
        .read_to_string()
        .map_err(|e| RetroError::Other(e.to_string()))?;

    // Parse <classification> blocks.
    let mut count = 0usize;
    let conn = state.db.lock().unwrap();
    let now = Utc::now().to_rfc3339();

    // Split into blocks.
    let mut rest = body.as_str();
    while let Some(start) = rest.find("<classification>") {
        rest = &rest[start..];
        let end = match rest.find("</classification>") {
            Some(e) => e + "</classification>".len(),
            None => break,
        };
        let block = &rest[..end];
        rest = &rest[end..];

        let id = extract_tag(block, "id").unwrap_or_default();
        let nomcourt = extract_tag(block, "nomcourt").unwrap_or_default();
        if id.is_empty() || nomcourt.is_empty() {
            continue;
        }
        // Split organism:value on first colon.
        let (system_name, value) = match nomcourt.split_once(':') {
            Some((s, v)) => (s.trim().to_string(), v.trim().to_string()),
            None => (nomcourt.clone(), nomcourt.clone()),
        };
        if system_name.is_empty() {
            continue;
        }

        // Upsert system by name.
        let sys_id: String = match conn.query_row(
            "SELECT id FROM classification_systems WHERE name=?1",
            [system_name.clone()],
            |r| r.get(0),
        ) {
            Ok(id) => id,
            Err(_) => {
                let id = Uuid::new_v4().to_string();
                let _ = conn.execute(
                    "INSERT INTO classification_systems (id,name,ss_id,created_at,updated_at) VALUES (?1,?2,NULL,?3,?4)",
                    params![id, system_name, now, now],
                );
                id
            }
        };

        // Upsert classification by ss_id (or system+value).
        let existing: Option<String> = if !id.is_empty() {
            conn.query_row(
                "SELECT id FROM classifications WHERE ss_id=?1",
                [id.clone()],
                |r| r.get(0),
            )
            .ok()
        } else {
            None
        };
        let cls_id = match existing {
            Some(i) => i,
            None => {
                // try system+value
                match conn.query_row(
                    "SELECT id FROM classifications WHERE system_id=?1 AND value=?2",
                    params![sys_id, value],
                    |r| r.get(0),
                ) {
                    Ok(i) => i,
                    Err(_) => {
                        let newid = Uuid::new_v4().to_string();
                        let _ = conn.execute(
                            "INSERT INTO classifications (id,system_id,value,ss_id,description,created_at,updated_at) VALUES (?1,?2,?3,?4,NULL,?5,?6)",
                            params![newid, sys_id, value, id, now, now],
                        );
                        newid
                    }
                }
            }
        };
        // Update ss_id if missing
        let _ = conn.execute(
            "UPDATE classifications SET ss_id=COALESCE(ss_id,?1), updated_at=?2 WHERE id=?3",
            params![id, now, cls_id],
        );
        count += 1;
    }

    Ok(count)
}

fn extract_tag(block: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let s = block.find(&open)? + open.len();
    let e = block[s..].find(&close)? + s;
    Some(block[s..e].trim().to_string())
}
