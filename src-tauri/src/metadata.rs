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
}

pub trait MetadataProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn fetch(&self, title: &str, platform: Option<&str>) -> Result<MetadataResult>;
}

#[derive(Clone)]
pub struct ScreenScraperProvider {
    pub cfg: crate::config::ProviderConfig,
}

impl MetadataProvider for ScreenScraperProvider {
    fn name(&self) -> &'static str {
        "screenscraper"
    }

    fn fetch(&self, title: &str, platform: Option<&str>) -> Result<MetadataResult> {
        // ScreenScraper jeuInfos
        let mut url = format!(
            "https://www.screenscraper.fr/api2/jeuInfos.php?devid={}&devpassword={}&softname=RETROVERSE&output=json&ssid={}&sspassword={}",
            urlencoding(&self.cfg.screenscraper_devid),
            urlencoding(&self.cfg.screenscraper_devpassword),
            urlencoding(&self.cfg.screenscraper_ssid),
            urlencoding(&self.cfg.screenscraper_sspassword),
        );

        let _ = platform;
        url.push_str(&format!("&recherche={}", urlencoding(title)));
        let resp = ureq::get(&url).call().map_err(|e| RetroError::Other(e.to_string()))?;
        let mut r = resp;
        let json: serde_json::Value = r.body_mut().read_json().map_err(|e| RetroError::Other(e.to_string()))?;
        // parse
        let jeu = &json["response"]["jeu"];
        let description = jeu["synopsis"]
            .as_array()
            .and_then(|a| a.iter().find(|s| s["langue"].as_str() == Some("fr") || s["langue"].as_str() == Some("en")))
            .and_then(|s| s["texte"].as_str())
            .map(|s| s.to_string())
            .or_else(|| jeu["synopsis"].as_array().and_then(|a| a.first()).and_then(|s| s["texte"].as_str()).map(|s| s.to_string()));
        let cover_url = jeu["medias"]
            .as_array()
            .and_then(|a| a.iter().find(|m| m["type"].as_str() == Some("box-2D") || m["type"].as_str() == Some("box-3D")))
            .and_then(|m| m["url"].as_str())
            .map(|s| s.to_string());
        let developer = jeu["developpeur"]["text"].as_str().map(|s| s.to_string())
            .or_else(|| jeu["developpeur"].as_str().map(|s| s.to_string()));
        let publisher = jeu["editeur"]["text"].as_str().map(|s| s.to_string())
            .or_else(|| jeu["editeur"].as_str().map(|s| s.to_string()));
        Ok(MetadataResult {
            title: jeu["noms"].as_array().and_then(|a| a.first()).and_then(|n| n["texte"].as_str()).map(|s| s.to_string()),
            description,
            cover_url,
            developer,
            publisher,
            ..Default::default()
        })
    }
}

#[derive(Clone)]
pub struct TheGamesDbProvider {
    pub cfg: crate::config::ProviderConfig,
}

impl MetadataProvider for TheGamesDbProvider {
    fn name(&self) -> &'static str {
        "thegamesdb"
    }

    fn fetch(&self, title: &str, platform: Option<&str>) -> Result<MetadataResult> {
        let url = format!(
            "https://api.thegamesdb.net/v1/Games/ByGameName?apikey={}&name={}&fields=players,publishers,genres,overview,rating",
            urlencoding(&self.cfg.tgdb_apikey),
            urlencoding(title),
        );
        if let Some(p) = platform {
            // platform id mapping not available; skip
            let _ = p;
        }
        let resp = ureq::get(&url).call().map_err(|e| RetroError::Other(e.to_string()))?;
        let mut r = resp;
        let json: serde_json::Value = r.body_mut().read_json().map_err(|e| RetroError::Other(e.to_string()))?;
        let game = json["data"]["games"]
            .as_array()
            .and_then(|a| a.first());
        let g = match game {
            Some(g) => g,
            None => return Ok(MetadataResult::default()),
        };
        let description = g["overview"].as_str().map(|s| s.to_string());
        let developer = g["developers"].as_array().and_then(|a| a.first()).and_then(|v| v.as_str()).map(|s| s.to_string());
        let publisher = g["publishers"].as_array().and_then(|a| a.first()).and_then(|v| v.as_str()).map(|s| s.to_string());
        let cover_url = json["include"]["boxart"]["data"]
            .as_object()
            .and_then(|o| o.values().next())
            .and_then(|a| a.as_array())
            .and_then(|a| a.first())
            .and_then(|b| b["url"].as_str())
            .map(|s| format!("https://cdn.thegamesdb.net/images/original/{}", s));
        Ok(MetadataResult {
            title: g["game_title"].as_str().map(|s| s.to_string()),
            description,
            cover_url,
            developer,
            publisher,
            ..Default::default()
        })
    }
}

fn urlencoding(s: &str) -> String {
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

pub fn providers_with_config(cfg: &crate::config::ProviderConfig) -> Vec<Arc<dyn MetadataProvider>> {
    let mut v: Vec<Arc<dyn MetadataProvider>> = Vec::new();
    if !cfg.screenscraper_devid.is_empty() {
        v.push(Arc::new(ScreenScraperProvider {
            cfg: cfg.clone(),
        }));
    }
    if !cfg.tgdb_apikey.is_empty() {
        v.push(Arc::new(TheGamesDbProvider {
            cfg: cfg.clone(),
        }));
    }
    v
}

pub fn provider_by_name(name: &str) -> Option<Arc<dyn MetadataProvider>> {
    provider_by_name_with_config(name, &crate::config::load())
}

pub fn provider_by_name_with_config(name: &str, cfg: &crate::config::ProviderConfig) -> Option<Arc<dyn MetadataProvider>> {
    providers_with_config(cfg).into_iter().find(|p| p.name() == name)
}
