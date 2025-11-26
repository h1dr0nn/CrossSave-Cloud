import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";

export interface FsEventPayload {
  path: string;
  kind: string;
  timestamp?: string;
}

export interface SaveMetadata {
  game_id: string;
  emulator_id: string;
  timestamp: number;
  version_id: string;
  file_list: string[];
  hash: string;
}

export interface PackagedSave {
  archive_path: string;
  metadata: SaveMetadata;
}

export interface HistoryEntry {
  archive_path: string;
  metadata_path: string;
  metadata: SaveMetadata;
}

export interface EmulatorProfile {
  emulator_id: string;
  name: string;
  default_save_paths: string[];
  file_patterns: string[];
}

export interface AppSettings {
  retention_limit: number;
  auto_delete: boolean;
}

export interface StorageInfo {
  history_path: string;
  size_bytes: number;
  total_size_bytes: number;
  total_versions: number;
  retention_bounds: [number, number];
}

export interface PackageResponse {
  packaged: PackagedSave;
  history: HistoryEntry;
}

export function packageGame(
  emulatorId: string,
  gameId: string
): Promise<PackageResponse> {
  return invoke("package_game", { emulator_id: emulatorId, game_id: gameId });
}

export function startWatcher(paths: string[]): Promise<void> {
  return invoke("start_watcher", { paths });
}

export function stopWatcher(): Promise<void> {
  return invoke("stop_watcher");
}

export function subscribeFsEvents(
  handler: (payload: FsEventPayload, event: Event<FsEventPayload>) => void
): Promise<UnlistenFn> {
  return listen<FsEventPayload>("watcher://fs-event", (event) => handler(event.payload, event));
}

export function packageSave(
  gameId: string,
  emulatorId: string,
  paths: string[],
  patterns: string[]
): Promise<PackageResponse> {
  return invoke("package_save", {
    game_id: gameId,
    emulator_id: emulatorId,
    paths,
    patterns
  });
}

export function listGamesFromHistory(): Promise<string[]> {
  return invoke("list_games_from_history");
}

export function listProfiles(): Promise<EmulatorProfile[]> {
  return invoke("list_profiles");
}

export function saveProfile(profile: EmulatorProfile): Promise<EmulatorProfile> {
  return invoke("save_profile", { profile });
}

export function deleteProfile(emulatorId: string): Promise<void> {
  return invoke("delete_profile", { emulator_id: emulatorId });
}

export function validatePaths(paths: string[]): Promise<string[]> {
  return invoke("validate_paths", { paths });
}

export function listHistory(gameId: string): Promise<HistoryEntry[]> {
  return invoke("list_history", { game_id: gameId });
}

export function getHistoryItem(gameId: string, versionId: string): Promise<HistoryEntry> {
  return invoke("get_history_item", { game_id: gameId, version_id: versionId });
}

export function rollbackVersion(gameId: string, versionId: string): Promise<PackagedSave> {
  return invoke("rollback_version", { game_id: gameId, version_id: versionId });
}

export function deleteHistoryItem(gameId: string, versionId: string): Promise<void> {
  return invoke("delete_history_item", { game_id: gameId, version_id: versionId });
}

export function getAppSettings(): Promise<AppSettings> {
  return invoke("get_app_settings");
}

export function updateAppSettings(settings: AppSettings): Promise<AppSettings> {
  return invoke("update_app_settings", { settings });
}

export function getStorageInfo(): Promise<StorageInfo> {
  return invoke("get_storage_info");
}

export function clearHistoryCache(): Promise<void> {
  return invoke("clear_history_cache");
}
export interface ScannedFile {
  path: string;
  name: string;
  size: number;
  modified: number;
}

export function scanSaveFiles(emulatorId: string): Promise<ScannedFile[]> {
  return invoke("scan_save_files", { emulatorId });
}

export interface PathStatus {
  path: string;
  exists: boolean;
  is_dir: boolean;
  error?: string;
}

export function checkPathStatus(emulatorId: string): Promise<PathStatus[]> {
  return invoke("check_path_status", { emulatorId });
}

export function openFolder(path: string): Promise<void> {
  return invoke("open_folder", { path });
}

export function selectDirectory(): Promise<string | null> {
  return invoke("select_directory");
}

