use crate::models::{GameFile, Platform};
use rusqlite::Connection;
use std::path::Path;

pub fn load_platforms(conn: &Connection) -> crate::error::Result<Vec<Platform>> {
    let mut stmt = conn.prepare(
        "SELECT id,name,slug,manufacturer,generation,release_year,extensions FROM platforms",
    )?;
    let rows = stmt.query_map([], |r| {
        let exts: String = r.get(6)?;
        let v: Vec<String> = serde_json::from_str(&exts).unwrap_or_default();
        Ok(Platform {
            id: r.get(0)?,
            name: r.get(1)?,
            slug: r.get(2)?,
            manufacturer: r.get(3)?,
            generation: r.get(4)?,
            release_year: r.get(5)?,
            extensions: v,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn guess_platform(ext: &str, platforms: &[Platform]) -> Option<String> {
    let e = format!(".{}", ext.to_lowercase());
    // Prefer exact single-extension matches; ambiguous ones resolved later
    let mut candidates: Vec<&Platform> = platforms
        .iter()
        .filter(|p| p.extensions.iter().any(|x| x.eq_ignore_ascii_case(&e)))
        .collect();
    if candidates.is_empty() {
        return None;
    }
    if candidates.len() == 1 {
        return Some(candidates[0].id.clone());
    }
    // Ambiguous: prefer by specificity (fewer extensions) then name
    candidates.sort_by_key(|p| (p.extensions.len(), p.name.clone()));
    Some(candidates[0].id.clone())
}

pub fn clean_title(filename: &str) -> String {
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);
    // Remove common tags in parentheses/brackets
    let mut s = stem.to_string();
    // Remove bracketed tags
    let mut out = String::new();
    let mut depth = 0;
    for c in s.chars() {
        match c {
            '(' | '[' | '{' => {
                depth += 1;
            }
            ')' | ']' | '}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ => {
                if depth == 0 {
                    out.push(c);
                }
            }
        }
    }
    s = out;
    // Collapse multiple spaces
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    // Remove trailing region codes like (U), (E) already removed; handle leading numbering
    s.trim().to_string()
}

pub fn identify_file(file: &GameFile, platforms: &[Platform]) -> (String, f64) {
    // Returns (title, confidence)
    let title = clean_title(&file.filename);
    let conf = if file.platform_id.is_some() { 0.7 } else { 0.4 };
    let _ = platforms;
    (title, conf)
}

pub fn ext_of(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_title_removes_tags() {
        assert_eq!(
            clean_title("Super Mario World (USA).sfc"),
            "Super Mario World"
        );
        assert_eq!(clean_title("Zelda [Hack].nes"), "Zelda");
        assert_eq!(clean_title("Game (U) (V1.0).bin"), "Game");
    }

    #[test]
    fn guess_platform_exact() {
        let platforms = vec![Platform {
            id: "snes".into(),
            name: "SNES".into(),
            slug: "snes".into(),
            manufacturer: None,
            generation: None,
            release_year: None,
            extensions: vec![".sfc".into(), ".smc".into()],
        }];
        assert_eq!(guess_platform("sfc", &platforms), Some("snes".into()));
        assert_eq!(guess_platform("txt", &platforms), None);
    }
}
