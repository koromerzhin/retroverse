import { invoke } from "@tauri-apps/api/core";

export const api = {
  listSources: () => invoke<any[]>("list_sources"),
  addSource: (name: string, path: string) => invoke<any>("add_source", { name, path }),
  removeSource: (id: string) => invoke<void>("remove_source", { id }),
  scanSource: (sourceId: string) => invoke<any>("scan_source", { sourceId }),
  testRemoteConnection: (host: string, port: number, username: string, keyPath: string) =>
    invoke<string>("test_remote_connection", { host, port, username, keyPath }),
  addRemoteSource: (name: string, host: string, port: number, username: string, keyPath: string, remotePath: string) =>
    invoke<any>("add_remote_source", { name, host, port, username, keyPath, remotePath }),
  listRemoteDir: (sourceId: string, path?: string) =>
    invoke<any[]>("list_remote_dir", { sourceId, path }),
  scanRemoteSource: (sourceId: string) => invoke<any>("scan_remote_source", { sourceId }),
  copyRemoteFiles: (sourceId: string, fileIds: string[], destDir: string) =>
    invoke<any>("copy_remote_files", { sourceId, fileIds, destDir }),
  listGames: (platform?: string, search?: string) => invoke<any[]>("list_games", { platform, search }),
  getGame: (id: string) => invoke<any>("get_game", { id }),
  correctTitle: (gameId: string, title: string) => invoke<void>("correct_game_title", { gameId, title }),
  linkFile: (fileId: string, gameId: string) => invoke<void>("link_file_to_game", { fileId, gameId }),
  stats: () => invoke<any>("stats"),
  listPlatforms: () => invoke<any[]>("list_platforms"),
  appInfo: () => invoke<any>("app_info"),
};

export async function pickDirectory(): Promise<string | null> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const res = await open({ directory: true, multiple: false });
  if (typeof res === "string") return res;
  return null;
}