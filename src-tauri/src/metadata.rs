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

/// Wikipedia-based provider (free, no API key).
pub struct WikipediaProvider;

impl MetadataProvider for WikipediaProvider {
    fn name(&self) -> &'static str {
        "wikipedia"
    }

    fn fetch(&self, title: &str, _platform: Option<&str>) -> Result<MetadataResult> {
        // Search Wikipedia
        let search_url = format!(
            "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&format=json&srlimit=1",
            urlencoding(title)
        );
        let mut resp = ureq::get(&search_url)
            .call()
            .map_err(|e| RetroError::Other(e.to_string()))?;
        let json: serde_json::Value = resp
            .body_mut()
            .read_json()
            .map_err(|e| RetroError::Other(e.to_string()))?;

        let page_title = json["query"]["search"]
            .get(0)
            .and_then(|v| v["title"].as_str())
            .map(|s| s.to_string());

        let page_title = match page_title {
            Some(t) => t,
            None => return Ok(MetadataResult::default()),
        };

        // Get extract + thumbnail
        let detail_url = format!(
            "https://en.wikipedia.org/w/api.php?action=query&prop=extracts|pageimages&exintro=1&explaintext=1&piprop=thumbnail&pithumbsize=500&titles={}&format=json",
            urlencoding(&page_title)
        );
        let mut resp = ureq::get(&detail_url)
            .call()
            .map_err(|e| RetroError::Other(e.to_string()))?;
        let json: serde_json::Value = resp
            .body_mut()
            .read_json()
            .map_err(|e| RetroError::Other(e.to_string()))?;

        let pages = &json["query"]["pages"];
        let page = pages.as_object().and_then(|o| o.values().next());

        let description = page
            .and_then(|p| p["extract"].as_str())
            .map(|s| s.to_string());
        let cover_url = page
            .and_then(|p| p["thumbnail"]["source"].as_str())
            .map(|s| s.to_string());

        Ok(MetadataResult {
            title: Some(page_title),
            description,
            cover_url,
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
    vec![Arc::new(WikipediaProvider)]
}

pub fn provider_by_name(name: &str) -> Option<Arc<dyn MetadataProvider>> {
    providers().into_iter().find(|p| p.name() == name)
}
