use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub manufacturer: Option<String>,
    pub generation: Option<i32>,
    pub release_year: Option<i32>,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSource {
    pub id: String,
    pub name: String,
    pub kind: String, // "local" | "remote"
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub last_scan_at: Option<DateTime<Utc>>,
    pub connection_id: Option<String>,
    pub remote_path: Option<String>,
    pub available: bool,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConnection {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub key_path: String,
    pub host_key_fingerprint: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: i64,
    pub modified_at: i64,
    pub extension: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResult {
    pub job_id: String,
    pub files: usize,
    pub bytes: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameFile {
    pub id: String,
    pub source_id: String,
    pub path: String,
    pub filename: String,
    pub extension: String,
    pub size: i64,
    pub modified_at: i64,
    pub hash: Option<String>,
    pub crc32: Option<String>,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha512: Option<String>,
    pub zip_inner: Option<String>,
    pub archive_hash: Option<String>,
    pub archive_crc32: Option<String>,
    pub archive_md5: Option<String>,
    pub archive_sha1: Option<String>,
    pub archive_sha512: Option<String>,
    pub platform_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub title: String,
    pub platform_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameFileLink {
    pub file_id: String,
    pub game_id: String,
    pub confidence: f64,
    pub manual: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredGame {
    pub game: Game,
    pub files: Vec<GameFile>,
    pub platform: Option<Platform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub source_id: String,
    pub scanned: usize,
    pub total: usize,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub source_id: String,
    pub files_found: usize,
    pub games_created: usize,
    pub games_updated: usize,
}

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Developer {
    pub id: String,
    pub name: String,
    pub ss_id: Option<String>,
    pub game_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publisher {
    pub id: String,
    pub name: String,
    pub ss_id: Option<String>,
    pub game_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    pub id: String,
    pub name_en: Option<String>,
    pub name_fr: Option<String>,
    pub name_de: Option<String>,
    pub name_es: Option<String>,
    pub name_it: Option<String>,
    pub name_pt: Option<String>,
    pub parent_id: Option<String>,
    pub ss_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationSystem {
    pub id: String,
    pub name: String,
    pub ss_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    pub id: String,
    pub system_id: String,
    pub system_name: Option<String>,
    pub value: String,
    pub ss_id: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
