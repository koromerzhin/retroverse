use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    pub screenscraper_devid: String,
    pub screenscraper_devpassword: String,
    pub screenscraper_softname: String,
    pub screenscraper_ssid: String,
    pub screenscraper_sspassword: String,
    pub tgdb_apikey: String,
}

fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("retroverse");
    let _ = fs::create_dir_all(&dir);
    dir.join("providers.json")
}

pub fn load() -> ProviderConfig {
    let path = config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        ProviderConfig::default()
    }
}

pub fn save(cfg: &ProviderConfig) -> Result<()> {
    let path = config_path();
    let content = serde_json::to_string_pretty(cfg).map_err(crate::error::RetroError::Serde)?;
    fs::write(&path, content).map_err(crate::error::RetroError::Io)?;
    Ok(())
}