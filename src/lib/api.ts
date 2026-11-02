import { invoke } from "@tauri-apps/api/core";

export const api = {
  listSources: () => invoke<any[]>("list_sources"),
  addSource: (name: string, path: string) => invoke<any>("add_source", { name, path }),
  removeSource: (id: string) => invoke<void>("remove_source", { id }),
  scanSource: (sourceId: string, folder?: string, platformFilter?: string) => invoke<any>("scan_source", { sourceId, folder, platformFilter }),
  scanLocalFolder: (sourceId: string, folderPath: string, platformFilter?: string) => invoke<any>("scan_local_folder", { sourceId, folderPath, platformFilter }),
  cancelScan: () => invoke<void>("cancel_scan"),
  testRemoteConnection: (host: string, port: number, username: string, keyPath: string) =>
    invoke<string>("test_remote_connection", { host, port, username, keyPath }),
  addRemoteSource: (name: string, host: string, port: number, username: string, keyPath: string, remotePath: string) =>
    invoke<any>("add_remote_source", { name, host, port, username, keyPath, remotePath }),
  listRemoteDir: (sourceId: string, path?: string) =>
    invoke<any[]>("list_remote_dir", { sourceId, path }),
  scanRemoteSource: (sourceId: string, folder?: string, platformFilter?: string) => invoke<any>("scan_remote_source", { sourceId, folder, platformFilter }),
  scanRemoteFolder: (sourceId: string, folderPath: string, platformFilter?: string) => invoke<any>("scan_remote_folder", { sourceId, folderPath, platformFilter }),
  copyRemoteFiles: (sourceId: string, fileIds: string[], destDir: string) =>
    invoke<any>("copy_remote_files", { sourceId, fileIds, destDir }),
  listGames: (platform?: string, search?: string, limit?: number, offset?: number) => invoke<any>("list_games", { platform, search, limit, offset }),
  getGame: (id: string) => invoke<any>("get_game", { id }),
  searchGamesFts: (query: string, platform?: string, limit?: number, offset?: number) => invoke<any>("search_games_fts", { query, platform, limit, offset }),
  platformGameCounts: () => invoke<any>("platform_game_counts"),
  findDuplicates: () => invoke<any[]>("find_duplicates"),
  listCollections: () => invoke<any[]>("list_collections"),
  createCollection: (name: string) => invoke<any>("create_collection", { name }),
  addToCollection: (collectionId: string, gameId: string) => invoke<void>("add_game_to_collection", { collectionId, gameId }),
  removeFromCollection: (collectionId: string, gameId: string) => invoke<void>("remove_game_from_collection", { collectionId, gameId }),
  collectionGames: (collectionId: string) => invoke<string[]>("collection_games", { collectionId }),
  correctTitle: (gameId: string, title: string) => invoke<void>("correct_game_title", { gameId, title }),
  linkFile: (fileId: string, gameId: string) => invoke<void>("link_file_to_game", { fileId, gameId }),
  stats: () => invoke<any>("stats"),
  listPlatforms: () => invoke<any[]>("list_platforms"),
  appInfo: () => invoke<any>("app_info"),
  resetDatabase: () => invoke<void>("reset_database"),
  clearLibrary: () => invoke<void>("clear_library"),
  getSystemLocale: () => invoke<string>("get_system_locale"),
  appReady: () => invoke<void>("app_ready"),
  getProviderConfig: () => invoke<any>("get_provider_config"),
  setProviderConfig: (cfg: any) => invoke<void>("set_provider_config", { cfg }),
  importGamelists: (sourceId: string) => invoke<number>("import_gamelists", { sourceId }),
  computeHashes: (sourceId?: string) => invoke<number>("compute_hashes", { sourceId }),
  computeHashesForGame: (gameId: string) => invoke<number>("compute_hashes_for_game", { gameId }),
  computeHashesForFile: (fileId: string) => invoke<number>("compute_hashes_for_file", { fileId }),
  exportBackup: (destPath: string) => invoke<string>("export_backup", { destPath }),
  importBackup: (srcPath: string) => invoke<number>("import_backup", { srcPath }),
  listTransferJobs: () => invoke<any[]>("list_transfer_jobs"),
  detectEmudeck: () => invoke<string | null>("detect_emudeck"),
  detectEmulators: () => invoke<string[]>("detect_running_emulators"),
  startSession: (gameId: string) => invoke<string>("start_session", { gameId }),
  stopSession: (sessionId: string) => invoke<number>("stop_session", { sessionId }),
  listSessions: (gameId?: string) => invoke<any[]>("list_sessions", { gameId }),
  sessionStats: () => invoke<any>("session_stats"),
  listMetadataProviders: () => invoke<string[]>("list_metadata_providers"),
  enrichGame: (gameId: string, provider: string) => invoke<any>("enrich_game", { gameId, provider }),
  getGameMetadata: (gameId: string) => invoke<any[]>("get_game_metadata", { gameId }),
  showNotification: (title: string, body: string) => invoke<void>("show_notification", { title, body }),
};

export async function pickDirectory(): Promise<string | null> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const res = await open({ directory: true, multiple: false });
  if (typeof res === "string") return res;
  return null;
}