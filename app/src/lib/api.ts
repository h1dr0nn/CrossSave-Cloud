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
): Promise<PackagedSave> {
  return invoke("package_save", {
    game_id: gameId,
    emulator_id: emulatorId,
    paths,
    patterns
  });
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
