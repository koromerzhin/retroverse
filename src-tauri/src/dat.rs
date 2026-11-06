use crate::error::Result;

#[derive(Debug, Clone)]
pub struct DatFile {
    pub name: String,
    pub system: Option<String>,
    pub version: Option<String>,
    pub entries: Vec<DatEntry>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DatEntry {
    pub name: String,
    pub size: Option<i64>,
    pub crc32: Option<String>,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
}

/// Parse a ClrMamePro-format DAT file (text).
pub fn parse_clrmamepro(content: &str) -> Result<DatFile> {
    let mut df = DatFile {
        name: "Unknown".into(),
        system: None,
        version: None,
        entries: Vec::new(),
    };
    let mut current: Option<DatEntry> = None;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("clrmamepro (")
            || line.starts_with("game (")
            || line.starts_with("rom (")
        {
            // handle game/rom
            if line.starts_with("game (") {
                if let Some(e) = current.take() {
                    df.entries.push(e);
                }
                current = Some(DatEntry {
                    name: String::new(),
                    size: None,
                    crc32: None,
                    md5: None,
                    sha1: None,
                    sha256: None,
                });
            }
            continue;
        }
        if line == ")" {
            if let Some(e) = current.take()
                && !e.name.is_empty()
            {
                df.entries.push(e);
            }
            continue;
        }
        // key value pairs: name "x", size 123, crc ABCD, md5 ..., sha1 ..., sha256 ...
        if let Some((k, v)) = line.split_once(' ') {
            let key = k.trim();
            let val = v.trim().trim_matches('"').to_string();
            match key {
                "name" => {
                    if let Some(e) = current.as_mut() {
                        e.name = val;
                    } else {
                        df.name = val;
                    }
                }
                "description" => {
                    if df.name == "Unknown" {
                        df.name = val;
                    }
                }
                "version" => {
                    df.version = Some(val);
                }
                "system" => {
                    df.system = Some(val);
                }
                "size" => {
                    if let Some(e) = current.as_mut() {
                        e.size = val.parse().ok();
                    }
                }
                "crc" | "crc32" => {
                    if let Some(e) = current.as_mut() {
                        e.crc32 = Some(val);
                    }
                }
                "md5" => {
                    if let Some(e) = current.as_mut() {
                        e.md5 = Some(val);
                    }
                }
                "sha1" => {
                    if let Some(e) = current.as_mut() {
                        e.sha1 = Some(val);
                    }
                }
                "sha256" => {
                    if let Some(e) = current.as_mut() {
                        e.sha256 = Some(val);
                    }
                }
                _ => {}
            }
        }
    }
    if let Some(e) = current.take()
        && !e.name.is_empty()
    {
        df.entries.push(e);
    }
    Ok(df)
}

/// Parse a No-Intro/Redump XML DAT.
pub fn parse_xml_dat(content: &str) -> Result<DatFile> {
    // Minimal XML parsing without external crate: extract <game name="..."> and <rom .../>
    let mut df = DatFile {
        name: "Unknown".into(),
        system: None,
        version: None,
        entries: Vec::new(),
    };
    // name from header
    if let Some(i) = content.find("<header>") {
        let h = &content[i..];
        if let Some(n) = extract_attr(h, "name") {
            df.name = n;
        }
        if let Some(v) = extract_attr(h, "version") {
            df.version = Some(v);
        }
        if let Some(s) = extract_attr(h, "description") {
            df.system = Some(s);
        }
    }
    // games
    let mut rest = content;
    while let Some(gi) = rest.find("<game ") {
        rest = &rest[gi..];
        let gname = extract_attr(rest, "name").unwrap_or_default();
        // find rom tag
        if let Some(ri) = rest.find("<rom ") {
            let romtag_end = rest[ri..]
                .find("/>")
                .map(|e| ri + e + 2)
                .unwrap_or(rest.len());
            let romtag = &rest[ri..romtag_end];
            let entry_name = extract_attr(romtag, "name").unwrap_or(gname.clone());
            let size = extract_attr(romtag, "size").and_then(|s| s.parse().ok());
            let crc = extract_attr(romtag, "crc");
            let md5 = extract_attr(romtag, "md5");
            let sha1 = extract_attr(romtag, "sha1");
            let sha256 = extract_attr(romtag, "sha256");
            df.entries.push(DatEntry {
                name: entry_name,
                size,
                crc32: crc,
                md5,
                sha1,
                sha256,
            });
            rest = &rest[romtag_end..];
        } else {
            // advance
            if let Some(e) = rest.find("</game>") {
                rest = &rest[e + 7..];
            } else {
                break;
            }
        }
    }
    Ok(df)
}

fn extract_attr(s: &str, attr: &str) -> Option<String> {
    let key = format!("{}=\"", attr);
    let i = s.find(&key)?;
    let start = i + key.len();
    let end = s[start..].find('"')? + start;
    Some(s[start..end].to_string())
}

pub fn parse_dat(content: &str) -> Result<DatFile> {
    if content.trim_start().starts_with('<') {
        parse_xml_dat(content)
    } else {
        parse_clrmamepro(content)
    }
}
