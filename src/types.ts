export interface Platform {
  id: string;
  name: string;
  slug: string;
  manufacturer: string | null;
  generation: number | null;
  release_year: number | null;
  extensions: string[];
}

export interface GameSource {
  id: string;
  name: string;
  kind: string;
  path: string;
  created_at: string;
  last_scan_at: string | null;
  connection_id: string | null;
  remote_path: string | null;
  available: boolean;
  last_error: string | null;
}

export interface GameFile {
  id: string;
  source_id: string;
  path: string;
  filename: string;
  extension: string;
  size: number;
  modified_at: number;
  hash: string | null;
  crc32: string | null;
  md5: string | null;
  sha1: string | null;
  sha512: string | null;
  zip_inner: string | null;
  archive_hash: string | null;
  archive_crc32: string | null;
  archive_md5: string | null;
  archive_sha1: string | null;
  archive_sha512: string | null;
  platform_id: string | null;
  is_dir: boolean;
  created_at: string;
}

export interface Game {
  id: string;
  title: string;
  platform_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface LibraryEntry {
  game: Game;
  platform: Platform | null;
  files: GameFile[];
  file_count: number;
}

export interface ScanResult {
  source_id: string;
  files_found: number;
  games_created: number;
  games_updated: number;
}

export interface RemoteEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified_at: number;
  extension: string;
}

export interface TransferResult {
  job_id: string;
  files: number;
  bytes: number;
  errors: string[];
}

export interface Stats {
  games: number;
  files: number;
  sources: number;
  platforms: number;
}

export interface AppInfo {
  version: string;
  db_path: string;
}