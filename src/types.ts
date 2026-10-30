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
  platform_id: string | null;
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