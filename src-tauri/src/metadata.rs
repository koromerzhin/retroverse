use crate::error::{Result, RetroError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetadataResult {
    pub title: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub release_year: Option<i32>,
    pub genres: Vec<String>,
    pub players: Option<String>,
    pub rating: Option<String>,
    pub release_dates: Vec<(String, String)>,
    pub classifications: Vec<(String, String)>,
    pub alt_titles: Vec<(String, String)>,
    pub modes: Vec<String>,
    pub themes: Vec<String>,
    pub styles: Vec<String>,
    pub ss_id: Option<String>,
    pub notgame: Option<String>,
    pub systeme: Option<String>,
    pub topstaff: Option<String>,
    pub rotation: Option<String>,
    pub familles: Vec<String>,
    pub raw: Option<serde_json::Value>,
}

pub trait MetadataProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn fetch(&self, title: &str, platform: Option<&str>) -> Result<MetadataResult>;
}

#[derive(Clone)]
pub struct ScreenScraperProvider {
    pub cfg: crate::config::ProviderConfig,
}

fn screenscraper_system_id(platform: Option<&str>) -> Option<&'static str> {
    match platform {
        Some("nes") | Some("famicom") | Some("fds") => Some("3"),
        Some("snes") | Some("sfc") | Some("sufami") | Some("supergrafx") => Some("4"),
        Some("n64") => Some("14"),
        Some("gb") | Some("gbc") | Some("sgb") => Some("9"),
        Some("gba") => Some("12"),
        Some("genesis") | Some("megadrive") => Some("1"),
        Some("mastersystem") | Some("mark3") => Some("2"),
        Some("gamegear") => Some("11"),
        Some("psx") | Some("ps1") => Some("57"),
        Some("ps2") => Some("80"),
        Some("psp") => Some("61"),
        Some("dreamcast") => Some("37"),
        Some("saturn") => Some("23"),
        Some("neogeo") => Some("142"),
        Some("arcade") | Some("mame") | Some("fba") | Some("fbneo") | Some("cps")
        | Some("cps1") | Some("cps2") | Some("cps3") => Some("75"),
        Some("amiga") | Some("amiga600") | Some("amiga1200") | Some("amigacd32") | Some("cdtv") => {
            Some("64")
        }
        Some("amstradcpc") | Some("gx4000") => Some("65"),
        Some("atari2600") => Some("26"),
        Some("atari5200") => Some("27"),
        Some("atari7800") => Some("28"),
        Some("atarilynx") => Some("29"),
        Some("atarijaguar") => Some("30"),
        Some("c64") => Some("66"),
        Some("msx") | Some("msx1") | Some("msx2") | Some("msxturbor") => Some("113"),
        Some("pcengine") | Some("tg16") => Some("31"),
        Some("pcenginecd") | Some("tg-cd") => Some("114"),
        Some("scummvm") => Some("231"),
        Some("solarus") => Some("244"),
        _ => None,
    }
}

impl MetadataProvider for ScreenScraperProvider {
    fn name(&self) -> &'static str {
        "screenscraper"
    }

    fn fetch(&self, title: &str, platform: Option<&str>) -> Result<MetadataResult> {
        let sid = screenscraper_system_id(platform);
        // Search by name first.
        let mut search_url = format!(
            "https://www.screenscraper.fr/api2/jeuRecherche.php?devid={}&devpassword={}&softname=RETROVERSE&output=json&ssid={}&sspassword={}&recherche={}",
            urlencoding(&self.cfg.screenscraper_devid),
            urlencoding(&self.cfg.screenscraper_devpassword),
            urlencoding(&self.cfg.screenscraper_ssid),
            urlencoding(&self.cfg.screenscraper_sspassword),
            urlencoding(title),
        );
        if let Some(s) = sid {
            search_url.push_str(&format!("&systemeid={}", s));
        }
        let resp = ureq::get(&search_url)
            .call()
            .map_err(|e| RetroError::Other(e.to_string()))?;
        let mut r = resp;
        let json: serde_json::Value = r
            .body_mut()
            .read_json()
            .map_err(|e| RetroError::Other(e.to_string()))?;
        let gameid = json["response"]["jeux"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|g| g["id"].as_i64().or_else(|| g["id"].as_str()?.parse().ok()))
            .map(|n| n.to_string())
            .ok_or_else(|| RetroError::NotFound("game not found".into()))?;

        // Fetch full game info by gameid.
        let mut url = format!(
            "https://www.screenscraper.fr/api2/jeuInfos.php?devid={}&devpassword={}&softname=RETROVERSE&output=json&ssid={}&sspassword={}&gameid={}",
            urlencoding(&self.cfg.screenscraper_devid),
            urlencoding(&self.cfg.screenscraper_devpassword),
            urlencoding(&self.cfg.screenscraper_ssid),
            urlencoding(&self.cfg.screenscraper_sspassword),
            gameid,
        );
        if let Some(s) = sid {
            url.push_str(&format!("&systemeid={}", s));
        }
        let resp = ureq::get(&url)
            .call()
            .map_err(|e| RetroError::Other(e.to_string()))?;
        let mut r = resp;
        let json: serde_json::Value = r
            .body_mut()
            .read_json()
            .map_err(|e| RetroError::Other(e.to_string()))?;
        // parse
        let jeu = &json["response"]["jeu"];
        let description = jeu["synopsis"]
            .as_array()
            .and_then(|a| {
                a.iter().find(|s| {
                    s["langue"].as_str() == Some("fr") || s["langue"].as_str() == Some("en")
                })
            })
            .and_then(|s| s["text"].as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                jeu["synopsis"]
                    .as_array()
                    .and_then(|a| a.first())
                    .and_then(|s| s["text"].as_str())
                    .map(|s| s.to_string())
            });
        let cover_url = jeu["medias"]
            .as_array()
            .and_then(|a| {
                a.iter().find(|m| {
                    m["type"].as_str() == Some("box-2D") || m["type"].as_str() == Some("box-3D")
                })
            })
            .and_then(|m| m["url"].as_str())
            .map(|s| s.to_string());
        let developer = jeu["developpeur"]["text"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| jeu["developpeur"].as_str().map(|s| s.to_string()));
        let publisher = jeu["editeur"]["text"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| jeu["editeur"].as_str().map(|s| s.to_string()));
        let players = jeu["joueurs"]["text"].as_str().map(|s| s.to_string());
        let rating = jeu["note"]["text"].as_str().map(|s| s.to_string());
        let genres: Vec<String> = jeu["genres"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|g| {
                        g["noms"]
                            .as_array()
                            .and_then(|n| {
                                n.iter().find(|x| {
                                    x["langue"].as_str() == Some("en")
                                        || x["langue"].as_str() == Some("fr")
                                })
                            })
                            .and_then(|x| x["text"].as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();
        let release_dates: Vec<(String, String)> = jeu["dates"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|d| {
                        Some((
                            d["region"].as_str()?.to_string(),
                            d["text"].as_str()?.to_string(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let classifications: Vec<(String, String)> = jeu["classifications"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|c| {
                        Some((
                            c["type"].as_str()?.to_string(),
                            c["text"].as_str()?.to_string(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let alt_titles: Vec<(String, String)> = jeu["noms"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|n| {
                        Some((
                            n["region"].as_str()?.to_string(),
                            n["text"].as_str()?.to_string(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let modes: Vec<String> = jeu["modes"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|m| {
                        m["noms"]
                            .as_array()
                            .and_then(|n| {
                                n.iter().find(|x| {
                                    x["langue"].as_str() == Some("en")
                                        || x["langue"].as_str() == Some("fr")
                                })
                            })
                            .and_then(|x| x["text"].as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();
        let themes: Vec<String> = jeu["themes"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|t| {
                        t["noms"]
                            .as_array()
                            .and_then(|n| {
                                n.iter().find(|x| {
                                    x["langue"].as_str() == Some("en")
                                        || x["langue"].as_str() == Some("fr")
                                })
                            })
                            .and_then(|x| x["text"].as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();
        let styles: Vec<String> = jeu["styles"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|s| {
                        s["noms"]
                            .as_array()
                            .and_then(|n| {
                                n.iter().find(|x| {
                                    x["langue"].as_str() == Some("en")
                                        || x["langue"].as_str() == Some("fr")
                                })
                            })
                            .and_then(|x| x["text"].as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();
        let ss_id = jeu["id"].as_str().map(|s| s.to_string());
        let notgame = jeu["notgame"].as_str().map(|s| s.to_string());
        let systeme = jeu["systeme"]["text"].as_str().map(|s| s.to_string());
        let topstaff = jeu["topstaff"].as_str().map(|s| s.to_string());
        let rotation = jeu["rotation"].as_str().map(|s| s.to_string());
        let familles: Vec<String> = jeu["familles"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|f| {
                        f["noms"]
                            .as_array()
                            .and_then(|n| {
                                n.iter().find(|x| {
                                    x["langue"].as_str() == Some("en")
                                        || x["langue"].as_str() == Some("fr")
                                })
                            })
                            .and_then(|x| x["text"].as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(MetadataResult {
            title: jeu["noms"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|n| n["text"].as_str())
                .map(|s| s.to_string()),
            description,
            cover_url,
            developer,
            publisher,
            release_year: None,
            genres,
            players,
            rating,
            release_dates,
            classifications,
            alt_titles,
            modes,
            themes,
            styles,
            ss_id,
            notgame,
            systeme,
            topstaff,
            rotation,
            familles,
            raw: Some(jeu.clone()),
        })
    }
}

pub fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '&' => "%26".to_string(),
            '?' => "%3F".to_string(),
            '#' => "%23".to_string(),
            '+' => "%2B".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

pub fn providers() -> Vec<Arc<dyn MetadataProvider>> {
    providers_with_config(&crate::config::load())
}

pub fn providers_with_config(
    cfg: &crate::config::ProviderConfig,
) -> Vec<Arc<dyn MetadataProvider>> {
    let mut v: Vec<Arc<dyn MetadataProvider>> = Vec::new();
    if !cfg.screenscraper_devid.is_empty() {
        v.push(Arc::new(ScreenScraperProvider { cfg: cfg.clone() }));
    }
    v
}

pub fn provider_by_name(name: &str) -> Option<Arc<dyn MetadataProvider>> {
    provider_by_name_with_config(name, &crate::config::load())
}

pub fn provider_by_name_with_config(
    name: &str,
    cfg: &crate::config::ProviderConfig,
) -> Option<Arc<dyn MetadataProvider>> {
    providers_with_config(cfg)
        .into_iter()
        .find(|p| p.name() == name)
}
