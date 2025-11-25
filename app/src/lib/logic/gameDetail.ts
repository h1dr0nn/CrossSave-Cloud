import { writable, derived, get } from "svelte/store";
import {
  listHistory,
  listProfiles,
  packageGame,
  subscribeFsEvents,
  type FsEventPayload,
  type HistoryEntry,
} from "../api";
import { pushError, pushInfo } from "../notifications";
import { formatErrorMessage } from "../errorMessages";
import { settingsStore } from "../settingsStore";

interface WatcherEvent {
  timestamp: Date;
  kind: string;
  fileName: string;
  path: string;
}

export function createGameDetailLogic(gameId: string) {
  // State
  const history = writable<HistoryEntry[]>([]);
  const loading = writable(true);
  const reloading = writable(false);
  const packaging = writable(false);
  const watcherEvents = writable<WatcherEvent[]>([]);
  const changesDetected = writable(false);
  const autoPackageEnabled = writable(false);
  const trackedPatterns = writable<string[]>([]);

  let patternsLoadedFor = "";
  let unlistenWatcher: (() => void) | null = null;

  const AUTO_PACKAGE_STORAGE_KEY = `auto-package:${gameId}`;

  // Computed
  const latestEntry = derived(history, ($history) => $history[0] ?? null);

  // Actions
  async function loadHistory() {
    if (!gameId || gameId.trim() === "") {
      pushError("Invalid game ID");
      loading.set(false);
      return;
    }

    console.log(`[GameDetail] loadHistory started for: ${gameId}`);
    try {
      if (get(loading)) {
        history.set([]);
      }
      reloading.set(true);
      console.log("[GameDetail] reloading set to TRUE, starting 1s delay");
      const minTime = new Promise((resolve) => setTimeout(resolve, 1000));
      const [entries] = await Promise.all([listHistory(gameId), minTime]);
      console.log(`[GameDetail] loadHistory completed, found ${entries.length} entries`);
      history.set(
        [...entries].sort((a, b) => b.metadata.timestamp - a.metadata.timestamp)
      );
    } catch (error) {
      console.error("[GameDetail] loadHistory error:", error);
      pushError(formatErrorMessage(error));
    } finally {
      loading.set(false);
      reloading.set(false);
      console.log("[GameDetail] reloading set to FALSE");
    }
  }

  async function loadPatterns(emulatorId: string) {
    if (!emulatorId || patternsLoadedFor === emulatorId) return;

    try {
      const profiles = await listProfiles();
      const match = profiles.find((profile) => profile.emulator_id === emulatorId);
      if (match) {
        trackedPatterns.set([...match.file_patterns]);
        patternsLoadedFor = emulatorId;
      }
    } catch (error) {
      console.error("Failed to load patterns", error);
    }
  }

  async function packageNow(emulatorId: string, gameName?: string) {
    if (!emulatorId) {
      pushError("Missing emulator id for packaging");
      console.error("[PACKAGE] Missing emulatorId");
      return;
    }

    const actualGameId = gameName || gameId;
    console.log(`[PACKAGE] Starting package for game: ${actualGameId}, emulator: ${emulatorId}`);
    packaging.set(true);
    try {
      await packageGame(emulatorId, actualGameId);
      pushInfo("Packaging completed");
      console.log("[PACKAGE] Packaging completed successfully");
      await loadHistory();
      changesDetected.set(false);

      // Refresh storage info for Settings/Storage page
      settingsStore.refreshStorage().catch(err =>
        console.warn("[PACKAGE] Failed to refresh storage info:", err)
      );
    } catch (error) {
      console.error("[PACKAGE] Packaging failed:", error);
      pushError(formatErrorMessage(error));
    } finally {
      packaging.set(false);
    }
  }

  function hydrateAutoPackage() {
    if (typeof localStorage === "undefined") return;
    const stored = localStorage.getItem(AUTO_PACKAGE_STORAGE_KEY);
    autoPackageEnabled.set(stored === "true");

    // Subscribe to changes to persist
    autoPackageEnabled.subscribe((value) => {
      if (typeof localStorage !== "undefined") {
        localStorage.setItem(AUTO_PACKAGE_STORAGE_KEY, String(value));
      }
    });
  }

  // Watcher Logic
  function normalizeKind(kind: string) {
    if (!kind) return "unknown";
    const simplified = kind.toLowerCase();
    if (simplified.includes("create")) return "create";
    if (simplified.includes("remove") || simplified.includes("delete"))
      return "delete";
    if (simplified.includes("modify") || simplified.includes("write"))
      return "modify";
    return simplified;
  }

  function globMatch(path: string, pattern: string) {
    const escaped = pattern
      .replace(/[.+^${}()|[\]\\]/g, "\\$&")
      .replace(/\\\*\\\*/g, ".*")
      .replace(/\\\*/g, "[^/]*");
    const regex = new RegExp(`^${escaped}$`);
    return regex.test(path) || regex.test(path.split("/").pop() ?? "");
  }

  function pathMatches(path: string, patterns: string[]) {
    if (!patterns.length) return false;
    const normalizedPath = path.replaceAll("\\", "/");
    return patterns.some((pattern) => globMatch(normalizedPath, pattern));
  }

  function appendWatcherEvent(payload: FsEventPayload) {
    const kind = normalizeKind(payload.kind);
    const timestamp = payload.timestamp
      ? new Date(payload.timestamp)
      : new Date();
    const fileName = payload.path.split(/[/\\]/).pop() || payload.path;

    watcherEvents.update(events => [
      {
        timestamp,
        kind,
        fileName,
        path: payload.path,
      },
      ...events,
    ].slice(0, 50));

    if (pathMatches(payload.path, get(trackedPatterns))) {
      changesDetected.set(true);
      if (get(autoPackageEnabled) && !get(packaging)) {
        // We need emulatorId here. It's usually derived in the component.
        // We might need to pass it or store it.
        // For now, we'll rely on the component calling packageNow with the ID,
        // but wait, auto-package needs to call it automatically.
        // We need to know the emulatorId.
      }
    }
  }

  // We need a way to trigger auto-package with the correct emulatorId.
  // Maybe we store emulatorId in the logic?
  const emulatorIdStore = writable("");

  function setEmulatorId(id: string) {
    emulatorIdStore.set(id);
    loadPatterns(id);
  }

  async function startWatcherFeed() {
    try {
      unlistenWatcher = await subscribeFsEvents((payload) => {
        appendWatcherEvent(payload);

        // Handle auto-package here
        if (pathMatches(payload.path, get(trackedPatterns))) {
          if (get(autoPackageEnabled) && !get(packaging)) {
            const emuId = get(emulatorIdStore);
            if (emuId) {
              packageNow(emuId);
            }
          }
        }
      });
    } catch (error) {
      console.error("Failed to subscribe to watcher events", error);
    }
  }

  function cleanup() {
    unlistenWatcher?.();
  }

  return {
    // Stores
    history,
    loading,
    reloading,
    packaging,
    watcherEvents,
    changesDetected,
    autoPackageEnabled,
    trackedPatterns,
    latestEntry,
    emulatorId: emulatorIdStore,

    // Methods
    loadHistory,
    packageNow,
    hydrateAutoPackage,
    startWatcherFeed,
    cleanup,
    setEmulatorId
  };
}
