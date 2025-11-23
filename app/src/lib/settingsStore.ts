import { writable } from "svelte/store";

import {
  clearHistoryCache,
  deleteProfile,
  getAppSettings,
  getStorageInfo,
  listProfiles,
  saveProfile,
  updateAppSettings,
  type AppSettings,
  type EmulatorProfile,
  type StorageInfo
} from "./api";
import { pushError, pushInfo } from "./notifications";

export interface OverrideState {
  enabled: boolean;
  path: string;
}

export interface SettingsState {
  profiles: EmulatorProfile[];
  loadingProfiles: boolean;
  selectedProfileId: string | null;
  overrides: Record<string, OverrideState>;
  appSettings: AppSettings | null;
  storageInfo: StorageInfo | null;
  loadingStorage: boolean;
  savingSettings: boolean;
  savingProfile: boolean;
  retentionBounds: [number, number];
}

const STORAGE_KEY = "save-path-overrides";
const isBrowser = typeof window !== "undefined";

function readOverrides(): Record<string, OverrideState> {
  if (!isBrowser) return {};

  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    return JSON.parse(raw) as Record<string, OverrideState>;
  } catch (error) {
    console.warn("Unable to read overrides", error);
    return {};
  }
}

function persistOverrides(map: Record<string, OverrideState>) {
  if (!isBrowser) return;

  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(map));
  } catch (error) {
    console.warn("Unable to persist overrides", error);
  }
}

function createSettingsStore() {
  const { subscribe, update, set } = writable<SettingsState>({
    profiles: [],
    loadingProfiles: false,
    selectedProfileId: null,
    overrides: readOverrides(),
    appSettings: null,
    storageInfo: null,
    loadingStorage: false,
    savingSettings: false,
    savingProfile: false,
    retentionBounds: [5, 20]
  });

  async function loadProfiles() {
    update((state) => ({ ...state, loadingProfiles: state.profiles.length === 0 }));
    try {
      const profiles = await listProfiles();
      update((state) => ({
        ...state,
        profiles,
        loadingProfiles: false,
        selectedProfileId: state.selectedProfileId ?? profiles[0]?.emulator_id ?? null
      }));
    } catch (error) {
      update((state) => ({ ...state, loadingProfiles: false }));
      pushError(`Failed to load profiles: ${error}`);
    }
  }

  async function loadSettings() {
    try {
      const settings = await getAppSettings();
      update((state) => ({ ...state, appSettings: settings }));
    } catch (error) {
      pushError(`Failed to load settings: ${error}`);
    }
  }

  async function loadStorage() {
    update((state) => ({ ...state, loadingStorage: true }));
    try {
      const storageInfo = await getStorageInfo();
      update((state) => ({
        ...state,
        storageInfo,
        loadingStorage: false,
        retentionBounds: storageInfo.retention_bounds
      }));
    } catch (error) {
      update((state) => ({ ...state, loadingStorage: false }));
      pushError(`Failed to fetch storage info: ${error}`);
    }
  }

  async function saveProfileEntry(profile: EmulatorProfile) {
    update((state) => ({ ...state, savingProfile: true }));
    try {
      const saved = await saveProfile(profile);
      pushInfo(`Saved profile ${saved.name}`);
      await loadProfiles();
      update((state) => ({ ...state, selectedProfileId: saved.emulator_id, savingProfile: false }));
    } catch (error) {
      update((state) => ({ ...state, savingProfile: false }));
      pushError(`Failed to save profile: ${error}`);
    }
  }

  async function deleteProfileEntry(emulatorId: string) {
    update((state) => ({ ...state, savingProfile: true }));
    try {
      await deleteProfile(emulatorId);
      pushInfo(`Deleted profile ${emulatorId}`);
      await loadProfiles();
      update((state) => ({ ...state, savingProfile: false, selectedProfileId: null }));
    } catch (error) {
      update((state) => ({ ...state, savingProfile: false }));
      pushError(`Failed to delete profile: ${error}`);
    }
  }

  async function applySettings(settings: AppSettings) {
    update((state) => ({ ...state, savingSettings: true }));
    try {
      const updated = await updateAppSettings(settings);
      update((state) => ({ ...state, appSettings: updated, savingSettings: false }));
      pushInfo("Settings updated");
      await loadStorage();
    } catch (error) {
      update((state) => ({ ...state, savingSettings: false }));
      pushError(`Failed to update settings: ${error}`);
    }
  }

  async function clearHistory() {
    try {
      await clearHistoryCache();
      pushInfo("History cache cleared");
      await loadStorage();
    } catch (error) {
      pushError(`Unable to clear history cache: ${error}`);
    }
  }

  function setOverride(emulatorId: string, override: OverrideState) {
    update((state) => {
      const overrides = { ...state.overrides, [emulatorId]: override };
      persistOverrides(overrides);
      return { ...state, overrides };
    });
  }

  function setSelectedProfile(emulatorId: string | null) {
    update((state) => ({ ...state, selectedProfileId: emulatorId }));
  }

  async function loadAll() {
    await Promise.all([loadProfiles(), loadSettings(), loadStorage()]);
  }

  return {
    subscribe,
    load: loadAll,
    refreshProfiles: loadProfiles,
    saveProfile: saveProfileEntry,
    deleteProfile: deleteProfileEntry,
    applySettings,
    clearHistory,
    setOverride,
    setSelectedProfile,
    refreshStorage: loadStorage,
    setState: set
  };
}

export const settingsStore = createSettingsStore();
