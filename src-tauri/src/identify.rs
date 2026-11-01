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
        let p = r?;
        if p.slug == "desktop" || p.slug == "emulators" || p.slug == "epic" || p.slug == "steam" {
            continue;
        }
        out.push(p);
    }
    Ok(out)
}

pub fn guess_platform(ext: &str, platforms: &[Platform]) -> Option<String> {
    let e = format!(".{}", ext.to_lowercase());
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
    candidates.sort_by_key(|p| (p.extensions.len(), p.name.clone()));
    Some(candidates[0].id.clone())
}

/// Infer platform from path segments (e.g. .../roms/nes/... -> nes).
pub fn infer_platform_from_path(path: &str, platforms: &[Platform]) -> Option<String> {
    for seg in path.split('/') {
        let low = seg.to_lowercase();
        for p in platforms {
            if p.slug == low || p.id == low {
                return Some(p.id.clone());
            }
        }
    }
    None
}

/// Parse a systeminfo.txt file. Returns the list of supported extensions.
pub fn parse_systeminfo_txt(content: &str) -> Option<Vec<String>> {
    let mut exts = Vec::new();
    let mut in_exts = false;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            in_exts = false;
            continue;
        }
        if line.starts_with("Supported file extensions:") {
            in_exts = true;
            let after = line.strip_prefix("Supported file extensions:").unwrap_or("").trim();
            if !after.is_empty() {
                for e in after.split_whitespace() {
                    exts.push(e.to_lowercase());
                }
                in_exts = false;
            }
        } else if in_exts {
            let toks: Vec<&str> = line.split_whitespace().collect();
            if !toks.is_empty() && toks.iter().all(|t| t.starts_with('.')) {
                for e in toks {
                    exts.push(e.to_lowercase());
                }
            } else {
                in_exts = false;
            }
        }
    }
    if exts.is_empty() { None } else { Some(exts) }
}

use crc::Crc;
use md5::Context as Md5Context;
use sha1::{Digest as Sha1Digest, Sha1};
use sha2::{Digest as Sha2Digest, Sha256};
use std::io::Read;

/// Compute hashes for a file. For .zip, reads the first inner ROM entry.
/// Returns (sha256, crc32, md5, sha1, zip_inner).
pub type Hashes = (Option<String>, Option<String>, Option<String>, Option<String>, Option<String>);

/// Compute hashes from raw bytes (for remote files).
pub fn compute_hashes_from_bytes(data: &[u8], filename: &str, allowed_exts: &[String]) -> Hashes {
    let lower = filename.to_lowercase();
    // Try zip
    if lower.ends_with(".zip")
        && let Ok(mut archive) = zip::ZipArchive::new(std::io::Cursor::new(data))
    {
        let mut idx = None;
        for i in 0..archive.len() {
            if let Ok(e) = archive.by_index(i) {
                if e.is_dir() {
                    continue;
                }
                let name = e.name().to_string();
                let ext = std::path::Path::new(&name)
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|x| format!(".{}", x.to_lowercase()))
                    .unwrap_or_default();
                if allowed_exts.is_empty() || allowed_exts.iter().any(|a| a.eq_ignore_ascii_case(&ext)) {
                    idx = Some(i);
                    break;
                }
            }
        }
        if let Some(i) = idx
            && let Ok(mut entry) = archive.by_index(i)
        {
            let inner_name = entry.name().to_string();
            let mut buf = [0u8; 65536];
            let mut sha256 = Sha256::new();
            let mut md5h = Md5Context::new();
            let mut sha1h = Sha1::new();
            let crc = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
            let mut crc_digest = crc.digest();
            loop {
                match entry.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        sha256.update(&buf[..n]);
                        md5h.consume(&buf[..n]);
                        sha1h.update(&buf[..n]);
                        crc_digest.update(&buf[..n]);
                    }
                    Err(_) => break,
                }
            }
            return (
                Some(format!("{:x}", sha256.finalize())),
                Some(format!("{:08x}", crc_digest.finalize())),
                Some(md5h.finalize().iter().map(|b| format!("{:02x}", b)).collect()),
                Some(sha1h.finalize().iter().map(|b| format!("{:02x}", b)).collect()),
                Some(inner_name),
            );
        }
    }
    // Hash the bytes themselves
    let mut sha256 = Sha256::new();
    let mut md5h = Md5Context::new();
    let mut sha1h = Sha1::new();
    let crc = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
    let mut crc_digest = crc.digest();
    sha256.update(data);
    md5h.consume(data);
    sha1h.update(data);
    crc_digest.update(data);
    (
        Some(format!("{:x}", sha256.finalize())),
        Some(format!("{:08x}", crc_digest.finalize())),
        Some(md5h.finalize().iter().map(|b| format!("{:02x}", b)).collect()),
        Some(sha1h.finalize().iter().map(|b| format!("{:02x}", b)).collect()),
        None,
    )
}

pub fn compute_hashes(path: &str, allowed_exts: &[String]) -> Hashes {
    let lower = path.to_lowercase();
    // 7z
    if lower.ends_with(".7z") {
        let mut inner_name: Option<String> = None;
        let mut sha256 = Sha256::new();
        let mut md5h = Md5Context::new();
        let mut sha1h = Sha1::new();
        let crc = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
        let mut crc_digest = crc.digest();
        let mut found = false;
        let mut buf = [0u8; 65536];
        let _ = sevenz_rust::decompress_file_with_extract_fn(
            path,
            std::path::Path::new("/dev/null"),
            |entry, reader, _dest| {
                if found {
                    return Ok(false);
                }
                if entry.is_directory() {
                    return Ok(false);
                }
                let name = entry.name().to_string();
                let ext = std::path::Path::new(&name)
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|x| format!(".{}", x.to_lowercase()))
                    .unwrap_or_default();
                if !allowed_exts.is_empty()
                    && !allowed_exts.iter().any(|a| a.eq_ignore_ascii_case(&ext))
                {
                    return Ok(false);
                }
                inner_name = Some(name);
                found = true;
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            sha256.update(&buf[..n]);
                            md5h.consume(&buf[..n]);
                            sha1h.update(&buf[..n]);
                            crc_digest.update(&buf[..n]);
                        }
                        Err(_) => break,
                    }
                }
                Ok(false)
            },
        );
        if found {
            return (
                Some(format!("{:x}", sha256.finalize())),
                Some(format!("{:08x}", crc_digest.finalize())),
                Some(md5h.finalize().iter().map(|b| format!("{:02x}", b)).collect()),
                Some(sha1h.finalize().iter().map(|b| format!("{:02x}", b)).collect()),
                inner_name,
            );
        }
    }
    // read file and delegate (zip handled in bytes, normal file hashed directly)
    if let Ok(data) = std::fs::read(path) {
        let filename = std::path::Path::new(path).file_name().and_then(|s| s.to_str()).unwrap_or(path);
        if !lower.ends_with(".7z") {
            return compute_hashes_from_bytes(&data, filename, allowed_exts);
        }
    }
    (None, None, None, None, None)
}

pub fn parse_gamelist_paths(content: &str) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    // simple regex-free parse: find <path>...</path>
    let mut rest = content;
    while let Some(start) = rest.find("<path>") {
        rest = &rest[start + 6..];
        if let Some(end) = rest.find("</path>") {
            let p = rest[..end].trim();
            let base = std::path::Path::new(p)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(p)
                .to_lowercase();
            set.insert(base);
            rest = &rest[end + 7..];
        } else {
            break;
        }
    }
    set
}

/// Parse a systems.txt file (format "slug: Name" per line).
pub fn parse_systems_txt(content: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((slug, name)) = line.split_once(':') {
            let slug = slug.trim().to_lowercase();
            let name = name.trim().to_string();
            if !slug.is_empty() && !name.is_empty() {
                out.push((slug, name));
            }
        }
    }
    out
}

/// Ensure platforms exist for each (slug, name) from systems.txt.
pub fn ensure_platforms(conn: &rusqlite::Connection, systems: &[(String, String)]) -> crate::error::Result<()> {
    for (slug, name) in systems {
        if slug == "desktop" || slug == "emulators" || slug == "epic" || slug == "steam" {
            continue;
        }
        // Skip if a platform with this name already exists.
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM platforms WHERE name=?1",
                [name.clone()],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;
        if exists {
            continue;
        }
        let id = slug.clone();
        let exts = "[]";
        conn.execute(
            "INSERT OR IGNORE INTO platforms (id,name,slug,manufacturer,generation,release_year,extensions) VALUES (?1,?2,?3,NULL,NULL,NULL,?4)",
            rusqlite::params![id, name, slug, exts],
        )?;
    }
    Ok(())
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
