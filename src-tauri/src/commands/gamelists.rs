//! Commands for gamelists.

use crate::error::{Result, RetroError};
use crate::state::AppState;
use rusqlite::params;
use serde_json::json;
use tauri::State;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct GameListEntry {
    path: String,
    name: String,
    desc: Option<String>,
    developer: Option<String>,
    publisher: Option<String>,
    genre: Option<String>,
    players: Option<String>,
    rating: Option<String>,
    releasedate: Option<String>,
}

fn parse_gamelist(content: &str) -> Vec<GameListEntry> {
    let mut out = Vec::new();
    // split into <game>...</game>
    let mut rest = content;
    while let Some(s) = rest.find("<game>") {
        rest = &rest[s + 6..];
        if let Some(e) = rest.find("</game>") {
            let block = &rest[..e];
            rest = &rest[e + 7..];
            let get = |tag: &str| {
                let open = format!("<{}>", tag);
                let close = format!("</{}>", tag);
                if let Some(i) = block.find(&open) {
                    let after = &block[i + open.len()..];
                    if let Some(j) = after.find(&close) {
                        return Some(after[..j].trim().to_string());
                    }
                }
                None
            };
            let path = match get("path") {
                Some(p) if !p.is_empty() => p,
                _ => continue,
            };
            let name = match get("name") {
                Some(n) if !n.is_empty() => n,
                _ => continue,
            };
            out.push(GameListEntry {
                path,
                name,
                desc: get("desc"),
                developer: get("developer"),
                publisher: get("publisher"),
                genre: get("genre"),
                players: get("players"),
                rating: get("rating"),
                releasedate: get("releasedate"),
            });
        } else {
            break;
        }
    }
    out
}

/// Deduce the Emulation root directory (`.../Emulation`) from a source path.
///
/// Handles `.../Emulation`, `.../Emulation/`, `.../Emulation/roms` and
/// `.../Emulation/roms/<platform>`; falls back to the parent directory.
pub(crate) fn emulation_root_from_source_path(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return "/".to_string();
    }
    let is_abs = trimmed.starts_with('/');
    let parts: Vec<&str> = trimmed.split('/').filter(|s| !s.is_empty()).collect();
    let join = |upto: usize| -> String {
        let body = parts[..upto].join("/");
        if is_abs { format!("/{}", body) } else { body }
    };
    // `.../roms[/...]` -> everything before "roms"
    if let Some(idx) = parts.iter().rposition(|p| p.eq_ignore_ascii_case("roms")) {
        let root = join(idx);
        if !root.is_empty() {
            return root;
        }
    }
    // `.../Emulation[/...]` -> the Emulation directory itself
    if let Some(idx) = parts
        .iter()
        .rposition(|p| p.eq_ignore_ascii_case("emulation"))
    {
        return join(idx + 1);
    }
    // fallback: parent directory
    if parts.len() >= 2 {
        return join(parts.len() - 1);
    }
    trimmed.to_string()
}

pub async fn import_gamelists_inner(state: &AppState, source_id: String) -> Result<usize> {
    // get source info
    let (kind, path, remote_path, connection_id) = {
        let conn = state.db.lock().unwrap();
        let (kind, path, rp, cid): (String, String, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT kind, path, remote_path, connection_id FROM game_sources WHERE id=?1",
                [source_id.clone()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )?;
        (kind, path, rp, cid)
    };

    let base_path = remote_path.clone().unwrap_or_else(|| path.clone());
    let emu_root = emulation_root_from_source_path(&base_path);

    // Collect gamelist contents (local or remote)
    let mut gamelists: Vec<(String, String)> = Vec::new(); // (platform, content)

    if kind == "remote" {
        let cid = connection_id.ok_or_else(|| RetroError::Invalid("not remote".into()))?;
        let (host, port, username, key_path) = {
            let conn = state.db.lock().unwrap();
            conn.query_row(
                "SELECT host,port,username,key_path FROM remote_connections WHERE id=?1",
                [cid],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, i64>(1)? as u16,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                    ))
                },
            )?
        };
        let sess = crate::remote::connect(&host, port, &username, &key_path).await?;
        // list gamelists dir
        let gl_dir = format!("{}/saves/es-de/gamelists", emu_root);
        if let Ok(rd) = sess.sftp.read_dir(&gl_dir).await {
            let dirs: Vec<_> = rd.collect();
            for d in dirs {
                if !d.metadata().is_dir() {
                    continue;
                }
                let platform = d.file_name();
                let gp = format!(
                    "{}/saves/es-de/gamelists/{}/gamelist.xml",
                    emu_root, platform
                );
                if let Ok(mut f) = sess.sftp.open(&gp).await {
                    use tokio::io::AsyncReadExt;
                    let mut buf = Vec::new();
                    let _ = f.read_to_end(&mut buf).await;
                    if let Ok(content) = String::from_utf8(buf) {
                        gamelists.push((platform, content));
                    }
                }
            }
        }

        let _ = sess
            .session
            .disconnect(russh::Disconnect::ByApplication, "", "")
            .await;
    } else {
        let gl_dir = std::path::Path::new(&emu_root)
            .join("saves")
            .join("es-de")
            .join("gamelists");
        if gl_dir.is_dir()
            && let Ok(rd) = std::fs::read_dir(&gl_dir)
        {
            for e in rd.flatten() {
                if !e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let platform = e.file_name().to_string_lossy().to_string();
                let gp = e.path().join("gamelist.xml");
                if gp.is_file()
                    && let Ok(content) = std::fs::read_to_string(&gp)
                {
                    gamelists.push((platform, content));
                }
            }
        }
    }

    // Import (optimized with in-memory game lookup + transaction)
    let mut imported = 0usize;
    let mut conn = state.db.lock().unwrap();
    let tx = conn.transaction()?;

    // Pre-load existing games into a map for O(1) lookup.
    let mut existing_games: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    {
        let mut stmt = tx.prepare("SELECT id, title, platform_id FROM games")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        })?;
        for r in rows {
            let (id, title, pid) = r?;
            existing_games.insert(
                format!("{}|{}", title.to_lowercase(), pid.unwrap_or_default()),
                id,
            );
        }
    }
    // Pre-load existing metadata records into a set for O(1) lookup.
    let mut existing_meta: std::collections::HashSet<String> = std::collections::HashSet::new();
    {
        let mut stmt = tx.prepare("SELECT game_id, field FROM metadata_records")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        for r in rows {
            let (gid, field) = r?;
            existing_meta.insert(format!("{}|{}", gid, field));
        }
    }

    for (platform, content) in &gamelists {
        let entries = parse_gamelist(content);
        let platform_id: Option<String> = tx
            .query_row(
                "SELECT id FROM platforms WHERE slug=?1 OR id=?1",
                [platform.clone()],
                |r| r.get(0),
            )
            .ok();

        for e in &entries {
            let key = format!(
                "{}|{}",
                e.name.to_lowercase(),
                platform_id.clone().unwrap_or_default()
            );
            let game_id = if let Some(gid) = existing_games.get(&key) {
                gid.clone()
            } else {
                let gid = crate::models::new_id();
                tx.execute(
                    "INSERT INTO games (id,title,platform_id,created_at,updated_at) VALUES (?1,?2,?3,?4,?5)",
                    params![gid, e.name, platform_id, chrono::Utc::now().to_rfc3339(), chrono::Utc::now().to_rfc3339()],
                )?;
                existing_games.insert(key, gid.clone());
                imported += 1;
                gid
            };

            let now = chrono::Utc::now().to_rfc3339();
            let fields: Vec<(&str, Option<&String>)> = vec![
                ("description", e.desc.as_ref()),
                ("developer", e.developer.as_ref()),
                ("publisher", e.publisher.as_ref()),
                ("genre", e.genre.as_ref()),
                ("release_date", e.releasedate.as_ref()),
            ];
            for (field, val) in fields {
                if let Some(v) = val {
                    let key = format!("{}|{}", game_id, field);
                    if !existing_meta.contains(&key) {
                        let id = crate::models::new_id();
                        tx.execute(
                            "INSERT INTO metadata_records (id,game_id,provider,field,value,provenance,fetched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                            params![id, game_id, "es-de", field, v, "es-de", now],
                        )?;
                        existing_meta.insert(key);
                    }
                }
            }
            let _ = json!({});
        }
    }

    tx.commit()?;
    Ok(imported)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Read a ScreenScraper `{ "text": "..." }` field.
fn json_text(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key)?
        .get("text")?
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Pick the best localized string from a ScreenScraper `noms`/`synopsis` array.
fn json_localized(v: &serde_json::Value, langs: &[&str]) -> Option<String> {
    let arr = v.as_array()?;
    for want in langs {
        let found = arr
            .iter()
            .find(|n| n.get("langue").and_then(|x| x.as_str()) == Some(*want))
            .and_then(|n| n.get("text"))
            .and_then(|x| x.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        if found.is_some() {
            return found;
        }
    }
    arr.iter()
        .find_map(|n| n.get("text").and_then(|x| x.as_str()))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Top-level genres, localized, comma separated (`Puzzle, Action`).
fn json_genres(v: &serde_json::Value) -> Option<String> {
    let arr = v.get("genres")?.as_array()?;
    let mut out: Vec<String> = Vec::new();
    for top_only in [true, false] {
        for g in arr {
            let parent = g.get("parentid").and_then(|x| x.as_str()).unwrap_or("0");
            if top_only && parent != "0" {
                continue;
            }
            if let Some(noms) = g.get("noms")
                && let Some(t) = json_localized(noms, &["fr", "en"])
                && !out.contains(&t)
            {
                out.push(t);
            }
        }
        if !out.is_empty() {
            break;
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out.join(", "))
    }
}

/// Normalize a date (`1989-11-01`, `1990`) to the ES-DE format (`19891101T000000`).
fn normalize_es_de_date(raw: &str) -> Option<String> {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 4 {
        return None;
    }
    let year = &digits[0..4];
    let month = if digits.len() >= 6 {
        &digits[4..6]
    } else {
        "01"
    };
    let day = if digits.len() >= 8 {
        &digits[6..8]
    } else {
        "01"
    };
    Some(format!("{}{}{}T000000", year, month, day))
}

fn json_release_date(v: &serde_json::Value) -> Option<String> {
    let arr = v
        .get("dates")
        .or_else(|| v.get("release_dates"))?
        .as_array()?;
    let picked = ["eu", "wor", "us", "jp"]
        .iter()
        .find_map(|want| {
            arr.iter().find(|d| {
                d.get("region")
                    .and_then(|x| x.as_str())
                    .map(|r| r.eq_ignore_ascii_case(want))
                    .unwrap_or(false)
            })
        })
        .or_else(|| arr.first())?;
    normalize_es_de_date(picked.get("text")?.as_str()?)
}

/// ScreenScraper rates games out of 20; ES-DE expects a 0..1 ratio.
fn json_rating(v: &serde_json::Value) -> Option<String> {
    let raw = v
        .get("note")?
        .get("text")?
        .as_str()?
        .trim()
        .replace(',', ".");
    let value: f64 = raw.parse().ok()?;
    if value <= 0.0 || value > 20.0 {
        return None;
    }
    Some(format!("{:.2}", value / 20.0))
}

#[derive(Default, Clone)]
struct GameMeta {
    desc: Option<String>,
    developer: Option<String>,
    publisher: Option<String>,
    genre: Option<String>,
    players: Option<String>,
    rating: Option<String>,
    releasedate: Option<String>,
}

#[derive(Default, Clone)]
struct GameMedia {
    image: Option<String>,
    thumbnail: Option<String>,
    marquee: Option<String>,
    fanart: Option<String>,
}

/// Combine stored metadata records with the ScreenScraper JSON payload.
fn build_game_meta(
    json: Option<&serde_json::Value>,
    record: impl Fn(&str) -> Option<String>,
) -> GameMeta {
    let json = json.unwrap_or(&serde_json::Value::Null);
    GameMeta {
        desc: record("description").or_else(|| {
            json.get("synopsis")
                .and_then(|s| json_localized(s, &["fr", "en"]))
        }),
        developer: record("developer").or_else(|| json_text(json, "developpeur")),
        publisher: record("publisher").or_else(|| json_text(json, "editeur")),
        genre: record("genre").or_else(|| json_genres(json)),
        players: record("players").or_else(|| json_text(json, "joueurs")),
        rating: record("rating").or_else(|| json_rating(json)),
        releasedate: record("release_date")
            .and_then(|v| normalize_es_de_date(&v))
            .or_else(|| json_release_date(json)),
    }
}

/// Everything needed to build gamelists, loaded in five queries.
///
/// Loading per game is far too slow on large libraries (tens of thousands of
/// rows), so the whole source is materialized once and reused per platform.
#[derive(Default)]
struct GamelistData {
    /// platform id -> [(game id, title)]
    games: std::collections::HashMap<String, Vec<(String, String)>>,
    /// game id -> representative ROM path for this source
    paths: std::collections::HashMap<String, String>,
    /// game id -> field -> latest value
    records: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    /// game id -> ScreenScraper JSON
    json: std::collections::HashMap<String, serde_json::Value>,
    /// game id -> cached local media
    media: std::collections::HashMap<String, GameMedia>,
}

impl GamelistData {
    fn load(conn: &rusqlite::Connection, source_id: &str) -> Result<Self> {
        let mut data = Self::default();
        {
            let mut stmt = conn.prepare(
                "SELECT gf.platform_id, g.id, g.title FROM game_files gf \
                 JOIN game_file_links l ON l.file_id = gf.id \
                 JOIN games g ON g.id = l.game_id \
                 WHERE gf.source_id=?1 AND gf.platform_id IS NOT NULL \
                 GROUP BY gf.platform_id, g.id \
                 ORDER BY g.title COLLATE NOCASE",
            )?;
            let rows = stmt.query_map([source_id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?;
            for (platform_id, game_id, title) in rows.flatten() {
                data.games
                    .entry(platform_id)
                    .or_default()
                    .push((game_id, title));
            }
        }
        {
            let mut stmt = conn.prepare(
                "SELECT l.game_id, MIN(gf.path) FROM game_files gf \
                 JOIN game_file_links l ON l.file_id = gf.id \
                 WHERE gf.source_id=?1 GROUP BY l.game_id",
            )?;
            let rows = stmt.query_map([source_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?;
            for (game_id, path) in rows.flatten() {
                data.paths.insert(game_id, path);
            }
        }
        {
            // Most recent first: the first value seen for a field wins.
            let mut stmt = conn.prepare(
                "SELECT game_id, field, value FROM metadata_records \
                 WHERE game_id IN (SELECT l.game_id FROM game_file_links l \
                                   JOIN game_files gf ON gf.id = l.file_id WHERE gf.source_id=?1) \
                 ORDER BY fetched_at DESC",
            )?;
            let rows = stmt.query_map([source_id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?;
            for (game_id, field, value) in rows.flatten() {
                data.records
                    .entry(game_id)
                    .or_default()
                    .entry(field)
                    .or_insert(value);
            }
        }
        {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT g.id, g.metadata_json FROM games g \
                 JOIN game_file_links l ON l.game_id = g.id \
                 JOIN game_files gf ON gf.id = l.file_id \
                 WHERE gf.source_id=?1 AND g.metadata_json IS NOT NULL",
            )?;
            let rows = stmt.query_map([source_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?;
            for (game_id, raw) in rows.flatten() {
                if let Ok(value) = serde_json::from_str(&raw) {
                    data.json.insert(game_id, value);
                }
            }
        }
        {
            let mut stmt = conn.prepare(
                "SELECT m.game_id, m.kind, m.path FROM media_assets m \
                 WHERE m.path IS NOT NULL AND m.game_id IN \
                   (SELECT l.game_id FROM game_file_links l \
                    JOIN game_files gf ON gf.id = l.file_id WHERE gf.source_id=?1) \
                 ORDER BY m.cached_at DESC",
            )?;
            let rows = stmt.query_map([source_id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?;
            for (game_id, kind, path) in rows.flatten() {
                if path.is_empty() || !std::path::Path::new(&path).exists() {
                    continue;
                }
                let media = data.media.entry(game_id).or_default();
                match kind.as_str() {
                    "cover" | "boxart" => media.image.get_or_insert(path),
                    "screenshot" | "ss" => media.thumbnail.get_or_insert(path),
                    "logo" | "wheel" => media.marquee.get_or_insert(path),
                    "hero" | "fanart" => media.fanart.get_or_insert(path),
                    "icon" => media.thumbnail.get_or_insert(path),
                    _ => continue,
                };
            }
        }
        Ok(data)
    }

    fn meta_for(&self, game_id: &str) -> GameMeta {
        build_game_meta(self.json.get(game_id), |field| {
            self.records
                .get(game_id)
                .and_then(|fields| fields.get(field))
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        })
    }

    fn rows_for(&self, platform_id: &str, rom_dir: &str, with_media: bool) -> Vec<GamelistRow> {
        let Some(games) = self.games.get(platform_id) else {
            return Vec::new();
        };
        games
            .iter()
            .map(|(game_id, title)| {
                let rel_path = match self.paths.get(game_id) {
                    Some(path) => {
                        let real = if path.starts_with("sftp://") {
                            crate::identify::remote_path_from_url(path).to_string()
                        } else {
                            path.clone()
                        };
                        relative_rom_path(&real, rom_dir, title)
                    }
                    None => format!("./{}", title),
                };
                GamelistRow {
                    title: title.clone(),
                    rel_path,
                    meta: self.meta_for(game_id),
                    media: with_media.then(|| self.media.get(game_id).cloned().unwrap_or_default()),
                }
            })
            .collect()
    }
}
struct GamelistRow {
    title: String,
    rel_path: String,
    meta: GameMeta,
    media: Option<GameMedia>,
}

/// ROM path relative to the platform folder, as expected by ES-DE (`./Game.nds`).
fn relative_rom_path(file_path: &str, rom_dir: &str, fallback: &str) -> String {
    let file = file_path.trim_end_matches('/');
    let base = rom_dir.trim_end_matches('/');
    if !base.is_empty()
        && let Some(rest) = file.strip_prefix(base)
    {
        let rest = rest.trim_start_matches('/');
        if !rest.is_empty() {
            return format!("./{}", rest);
        }
    }
    let name = std::path::Path::new(file)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| fallback.to_string());
    format!("./{}", name)
}

fn game_block(row: &GamelistRow) -> String {
    let mut content = String::from("  <game>\n");
    content.push_str(&format!("    <path>{}</path>\n", xml_escape(&row.rel_path)));
    content.push_str(&format!("    <name>{}</name>\n", xml_escape(&row.title)));
    let fields: [(&str, &Option<String>); 7] = [
        ("desc", &row.meta.desc),
        ("developer", &row.meta.developer),
        ("publisher", &row.meta.publisher),
        ("genre", &row.meta.genre),
        ("players", &row.meta.players),
        ("rating", &row.meta.rating),
        ("releasedate", &row.meta.releasedate),
    ];
    for (tag, value) in fields {
        if let Some(v) = value {
            content.push_str(&format!("    <{}>{}</{}>\n", tag, xml_escape(v), tag));
        }
    }
    if let Some(media) = &row.media {
        let tags: [(&str, &Option<String>); 4] = [
            ("image", &media.image),
            ("thumbnail", &media.thumbnail),
            ("marquee", &media.marquee),
            ("fanart", &media.fanart),
        ];
        for (tag, value) in tags {
            if let Some(v) = value {
                content.push_str(&format!("    <{}>{}</{}>\n", tag, xml_escape(v), tag));
            }
        }
    }
    content.push_str("  </game>");
    content
}

fn block_tag(block: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = block.find(&open)?;
    let after = &block[start + open.len()..];
    let end = after.find(&close)?;
    Some(after[..end].trim().to_string())
}

fn block_has_tag(block: &str, tag: &str) -> bool {
    block.contains(&format!("<{}>", tag))
}

fn insert_tag(block: &str, tag: &str, value: &str) -> String {
    let line = format!("    <{}>{}</{}>\n", tag, value, tag);
    match block.rfind("</game>") {
        Some(idx) => format!("{}{}{}", &block[..idx], line, &block[idx..]),
        None => block.to_string(),
    }
}

fn parse_gamelist_blocks(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find("<game>") {
        let after = &rest[start..];
        let Some(end) = after.find("</game>") else {
            break;
        };
        let stop = end + "</game>".len();
        out.push(after[..stop].to_string());
        rest = &after[stop..];
    }
    out
}

/// Merge generated metadata into an existing gamelist without losing what
/// ES-DE owns (favorites, play counts, hidden flags, scraped media/videos).
fn merge_gamelist(rows: &[GamelistRow], existing: Option<&str>) -> String {
    let mut blocks: Vec<String> = Vec::new();
    let mut used = vec![false; rows.len()];
    if let Some(existing) = existing {
        for block in parse_gamelist_blocks(existing) {
            let matched = block_tag(&block, "path").and_then(|path| {
                rows.iter()
                    .position(|r| r.rel_path.eq_ignore_ascii_case(&path))
            });
            match matched {
                Some(idx) => {
                    used[idx] = true;
                    blocks.push(merged_block(&rows[idx], &block));
                }
                None => blocks.push(block),
            }
        }
    }
    for (idx, row) in rows.iter().enumerate() {
        if !used[idx] {
            blocks.push(game_block(row));
        }
    }
    let mut content = String::from("<?xml version=\"1.0\"?>\n<gameList>\n");
    for block in blocks {
        content.push_str(block.trim_end());
        content.push('\n');
    }
    content.push_str("</gameList>\n");
    content
}

fn merged_block(row: &GamelistRow, existing: &str) -> String {
    let mut block = game_block(row);
    // Keep everything we cannot provide: ES-DE state, scraped media, and
    // metadata already present in the file for games the library knows little
    // about.
    for tag in [
        "desc",
        "developer",
        "publisher",
        "genre",
        "players",
        "rating",
        "releasedate",
        "favorite",
        "playcount",
        "lastplayed",
        "hidden",
        "kidgame",
        "sortname",
        "video",
        "image",
        "thumbnail",
        "marquee",
        "fanart",
    ] {
        if !block_has_tag(&block, tag)
            && let Some(value) = block_tag(existing, tag)
            && !value.is_empty()
        {
            block = insert_tag(&block, tag, &value);
        }
    }
    block
}

#[derive(serde::Serialize)]
pub struct PlatformOption {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub games: i64,
}

#[derive(serde::Serialize)]
pub struct SourcePlatforms {
    pub root: String,
    pub gamelist_dir: String,
    pub platforms: Vec<PlatformOption>,
}

fn source_base_path(path: &str, remote_path: Option<String>) -> String {
    remote_path
        .filter(|p| !p.trim().is_empty())
        .unwrap_or_else(|| path.to_string())
}

fn gamelist_dir_for(base_path: &str) -> String {
    let root = emulation_root_from_source_path(base_path);
    format!("{}/saves/es-de/gamelists", root.trim_end_matches('/'))
}

/// True when the directory is literally named `Emulation`.
fn has_emulation_name(root: &str) -> bool {
    std::path::Path::new(root)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.eq_ignore_ascii_case("emulation"))
        .unwrap_or(false)
}

/// Safety net: never write outside an Emulation tree.
///
/// The root must either be named `Emulation` or contain a `roms` subfolder;
/// this prevents writing to e.g. `/run/media/deck/WDC1/saves/...` when the
/// source path is `/run/media/deck/WDC1/Emulation`.
fn emulation_root_is_valid_local(root: &str) -> bool {
    has_emulation_name(root) || std::path::Path::new(root).join("roms").is_dir()
}

fn invalid_root_error(root: &str) -> RetroError {
    RetroError::Invalid(format!(
        "racine Emulation introuvable pour « {} » : dossier attendu « .../Emulation » (avec un sous-dossier roms). Rien n'a été écrit.",
        root
    ))
}

fn resolve_platforms(
    conn: &rusqlite::Connection,
    source_id: &str,
    filter: Option<&str>,
) -> Result<Vec<(String, String)>> {
    if let Some(pf) = filter.map(str::trim).filter(|p| !p.is_empty()) {
        let found: Option<(String, String)> = conn
            .query_row(
                "SELECT id, slug FROM platforms WHERE id=?1 OR slug=?1",
                [pf],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();
        return Ok(found.into_iter().collect());
    }
    let mut stmt = conn.prepare(
        "SELECT DISTINCT p.id, p.slug FROM game_files gf JOIN platforms p ON p.id = gf.platform_id \
         WHERE gf.source_id=?1 ORDER BY p.name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([source_id], |r| Ok((r.get(0)?, r.get(1)?)))?;
    Ok(rows.flatten().collect())
}

fn platform_rom_dir(base_path: &str, emu_root: &str, slug: &str) -> String {
    let base = base_path.trim_end_matches('/');
    let last = base.rsplit('/').next().unwrap_or_default();
    if last.eq_ignore_ascii_case(slug) {
        return base.to_string();
    }
    format!("{}/roms/{}", emu_root.trim_end_matches('/'), slug)
}

async fn ensure_remote_dir(sftp: &russh_sftp::client::SftpSession, dir: &str) -> Result<()> {
    let mut current = String::new();
    for part in dir.split('/').filter(|s| !s.is_empty()) {
        current.push('/');
        current.push_str(part);
        if sftp.try_exists(current.clone()).await.unwrap_or(false) {
            continue;
        }
        sftp.create_dir(current.clone())
            .await
            .map_err(|e| RetroError::Other(format!("création dossier {}: {}", current, e)))?;
    }
    Ok(())
}

/// Overwrite a remote file (create + truncate + write).
///
/// `SftpSession::write` only sets `WRITE`, which would leave the tail of a
/// longer existing file behind, so the flags are spelled out here.
async fn write_remote_file(
    sftp: &russh_sftp::client::SftpSession,
    path: &str,
    data: &[u8],
) -> Result<()> {
    use russh_sftp::protocol::OpenFlags;
    use tokio::io::AsyncWriteExt;
    let mut file = sftp
        .open_with_flags(
            path.to_string(),
            OpenFlags::WRITE | OpenFlags::CREATE | OpenFlags::TRUNCATE,
        )
        .await
        .map_err(|e| RetroError::Other(format!("ouverture de {}: {}", path, e)))?;
    file.write_all(data)
        .await
        .map_err(|e| RetroError::Other(format!("écriture de {}: {}", path, e)))?;
    file.close()
        .await
        .map_err(|e| RetroError::Other(format!("fermeture de {}: {}", path, e)))?;
    Ok(())
}

#[tauri::command]
pub fn list_source_platforms(state: State<AppState>, source_id: String) -> Result<SourcePlatforms> {
    let conn = state.db.lock().unwrap();
    let (path, remote_path): (String, Option<String>) = conn.query_row(
        "SELECT path, remote_path FROM game_sources WHERE id=?1",
        [source_id.clone()],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let base = source_base_path(&path, remote_path);
    let mut stmt = conn.prepare(
        "SELECT p.id, p.slug, p.name, COUNT(DISTINCT g.id) \
         FROM game_files gf JOIN platforms p ON p.id = gf.platform_id \
         JOIN game_file_links l ON l.file_id = gf.id JOIN games g ON g.id = l.game_id \
         WHERE gf.source_id=?1 GROUP BY p.id ORDER BY p.name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([source_id], |r| {
        Ok(PlatformOption {
            id: r.get(0)?,
            slug: r.get(1)?,
            name: r.get(2)?,
            games: r.get(3)?,
        })
    })?;
    Ok(SourcePlatforms {
        root: emulation_root_from_source_path(&base),
        gamelist_dir: gamelist_dir_for(&base),
        platforms: rows.flatten().collect(),
    })
}

pub async fn generate_gamelist_inner(
    state: &AppState,
    source_id: String,
    platform_filter: Option<String>,
) -> Result<usize> {
    let (kind, path, remote_path, connection_id) = {
        let conn = state.db.lock().unwrap();
        conn.query_row(
            "SELECT kind, path, remote_path, connection_id FROM game_sources WHERE id=?1",
            [source_id.clone()],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<String>>(3)?,
                ))
            },
        )?
    };
    let base = source_base_path(&path, remote_path);
    let emu_root = emulation_root_from_source_path(&base);
    let gamelist_dir = gamelist_dir_for(&base);

    let outputs: Vec<(String, usize, Vec<GamelistRow>)> = {
        let conn = state.db.lock().unwrap();
        let platforms = resolve_platforms(&conn, &source_id, platform_filter.as_deref())?;
        let data = GamelistData::load(&conn, &source_id)?;
        let mut outputs = Vec::new();
        for (platform_id, slug) in platforms {
            let rom_dir = platform_rom_dir(&base, &emu_root, &slug);
            let rows = data.rows_for(&platform_id, &rom_dir, kind == "local");
            if rows.is_empty() {
                continue;
            }
            outputs.push((slug, rows.len(), rows));
        }
        outputs
    };

    let mut total = 0usize;
    if kind == "remote" {
        let cid = connection_id
            .ok_or_else(|| RetroError::Invalid("source distante sans connexion".into()))?;
        let (host, port, username, key_path) = {
            let conn = state.db.lock().unwrap();
            conn.query_row(
                "SELECT host, port, username, key_path FROM remote_connections WHERE id=?1",
                [cid],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, i64>(1)? as u16,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                    ))
                },
            )?
        };
        let sess = crate::remote::connect(&host, port, &username, &key_path).await?;
        let root_ok = has_emulation_name(&emu_root)
            || sess
                .sftp
                .try_exists(format!("{}/roms", emu_root.trim_end_matches('/')))
                .await
                .unwrap_or(false);
        if !root_ok {
            return Err(invalid_root_error(&emu_root));
        }
        for (slug, count, rows) in &outputs {
            let dir = format!("{}/{}", gamelist_dir, slug);
            ensure_remote_dir(&sess.sftp, &dir).await?;
            let dest = format!("{}/gamelist.xml", dir);
            let existing = sess
                .sftp
                .read(dest.clone())
                .await
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok());
            let merged = merge_gamelist(rows, existing.as_deref());
            if existing.as_deref() != Some(merged.as_str()) {
                write_remote_file(&sess.sftp, &dest, merged.as_bytes()).await?;
            }
            total += count;
        }
    } else {
        if !emulation_root_is_valid_local(&emu_root) {
            return Err(invalid_root_error(&emu_root));
        }
        for (slug, count, rows) in &outputs {
            let dir = std::path::Path::new(&gamelist_dir).join(slug);
            std::fs::create_dir_all(&dir).map_err(RetroError::Io)?;
            let dest = dir.join("gamelist.xml");
            let existing = std::fs::read_to_string(&dest).ok();
            let merged = merge_gamelist(rows, existing.as_deref());
            if existing.as_deref() != Some(merged.as_str()) {
                std::fs::write(&dest, merged).map_err(RetroError::Io)?;
            }
            total += count;
        }
    }

    Ok(total)
}

#[tauri::command]
pub async fn generate_gamelist(
    state: State<'_, AppState>,
    source_id: String,
    platform_id: Option<String>,
) -> Result<usize> {
    generate_gamelist_inner(&state, source_id, platform_id).await
}

#[tauri::command]
pub async fn import_gamelists(state: State<'_, AppState>, source_id: String) -> Result<usize> {
    import_gamelists_inner(&state, source_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emulation_root_from_roms_suffix() {
        assert_eq!(
            emulation_root_from_source_path("/home/u/Emulation/roms"),
            "/home/u/Emulation"
        );
        assert_eq!(
            emulation_root_from_source_path("/home/u/Emulation/roms/nds/"),
            "/home/u/Emulation"
        );
        assert_eq!(
            emulation_root_from_source_path("/run/media/deck/WDC1/Emulation/roms"),
            "/run/media/deck/WDC1/Emulation"
        );
    }

    #[test]
    fn emulation_root_from_emulation_suffix() {
        assert_eq!(
            emulation_root_from_source_path("/run/media/deck/WDC1/Emulation/"),
            "/run/media/deck/WDC1/Emulation"
        );
        assert_eq!(
            emulation_root_from_source_path("/home/u/Emulation"),
            "/home/u/Emulation"
        );
    }

    #[test]
    fn emulation_root_fallback_parent() {
        assert_eq!(emulation_root_from_source_path("/data/games"), "/data");
        assert_eq!(emulation_root_from_source_path("/"), "/");
    }

    #[test]
    fn deck_source_path_never_writes_beside_emulation() {
        // Source used by the Steam Deck remote source: the gamelists must land
        // in .../Emulation/saves/es-de/gamelists, never in .../WDC1/saves.
        assert_eq!(
            gamelist_dir_for("/run/media/deck/WDC1/Emulation/"),
            "/run/media/deck/WDC1/Emulation/saves/es-de/gamelists"
        );
        assert_eq!(
            gamelist_dir_for("/run/media/deck/WDC1/Emulation/roms"),
            "/run/media/deck/WDC1/Emulation/saves/es-de/gamelists"
        );
        assert!(!emulation_root_is_valid_local("/run/media/deck/WDC1"));
    }

    #[test]
    fn es_de_date_normalization() {
        assert_eq!(
            normalize_es_de_date("1989-11-01").as_deref(),
            Some("19891101T000000")
        );
        assert_eq!(
            normalize_es_de_date("1990").as_deref(),
            Some("19900101T000000")
        );
        assert_eq!(normalize_es_de_date(""), None);
    }

    #[test]
    fn relative_path_uses_platform_folder() {
        assert_eq!(
            relative_rom_path("/e/roms/nds/Mario.nds", "/e/roms/nds", "Mario"),
            "./Mario.nds"
        );
        assert_eq!(
            relative_rom_path("/e/roms/nds/sub/Mario.nds", "/e/roms/nds", "Mario"),
            "./sub/Mario.nds"
        );
        assert_eq!(
            relative_rom_path("/other/Mario.nds", "/e/roms/nds", "Mario"),
            "./Mario.nds"
        );
    }

    #[test]
    fn xml_special_chars_are_escaped() {
        assert_eq!(xml_escape("A & B <x>"), "A &amp; B &lt;x&gt;");
    }

    #[test]
    fn merge_preserves_es_de_owned_fields() {
        let rows = vec![GamelistRow {
            title: "Mario".into(),
            rel_path: "./Mario.nds".into(),
            meta: GameMeta {
                developer: Some("Nintendo".into()),
                ..Default::default()
            },
            media: None,
        }];
        let existing = "<?xml version=\"1.0\"?>\n<gameList>\n  <game>\n    \
<path>./Mario.nds</path>\n    <name>Old name</name>\n    <desc>Old desc</desc>\n    \
<favorite>true</favorite>\n    <playcount>7</playcount>\n    \
<image>./media/mario.png</image>\n  </game>\n  <game>\n    \
<path>./Other.nds</path>\n    <name>Other</name>\n  </game>\n</gameList>\n";
        let merged = merge_gamelist(&rows, Some(existing));
        assert!(merged.contains("<name>Mario</name>"));
        assert!(merged.contains("<developer>Nintendo</developer>"));
        // Metadata we do not have is kept from the existing file.
        assert!(merged.contains("<desc>Old desc</desc>"));
        assert!(merged.contains("<favorite>true</favorite>"));
        assert!(merged.contains("<playcount>7</playcount>"));
        assert!(merged.contains("<image>./media/mario.png</image>"));
        // ROMs unknown to the library are kept untouched.
        assert!(merged.contains("<path>./Other.nds</path>"));
        assert_eq!(merged.matches("<game>").count(), 2);
    }
}
