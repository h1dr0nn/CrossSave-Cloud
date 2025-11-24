import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Authentication state
interface AuthState {
    isLoggedIn: boolean;
    email: string | null;
}

// Sync status from backend
interface SyncStatus {
    queue_length: number;
    active_job: any | null;
    last_sync: string | null;
    is_syncing: boolean;
}

// Cloud version summary
interface CloudVersion {
    game_id: string;
    emulator_id: string;
    version_id: string;
    timestamp: number;
    size_bytes: number;
    hash: string;
    device_id: string;
}

// Create stores
const authState = writable<AuthState>({
    isLoggedIn: false,
    email: null
});

const syncStatus = writable<SyncStatus>({
    queue_length: 0,
    active_job: null,
    last_sync: null,
    is_syncing: false
});

const cloudVersions = writable<Map<string, CloudVersion[]>>(new Map());

// Listen to sync status events from backend
if (typeof window !== 'undefined') {
    listen<SyncStatus>('sync://status', (event) => {
        syncStatus.set(event.payload);
    }).catch(console.error);
}

// Actions
export const cloudStore = {
    // Subscribe to stores
    auth: { subscribe: authState.subscribe },
    syncStatus: { subscribe: syncStatus.subscribe },
    cloudVersions: { subscribe: cloudVersions.subscribe },

    // Auth actions
    async login(email: string, password: string): Promise<boolean> {
        // Mock login for now
        if (email && password) {
            authState.set({ isLoggedIn: true, email });

            // Enable cloud sync in backend
            try {
                const settings = await invoke<any>('get_app_settings');
                settings.cloud.enabled = true;
                await invoke('update_app_settings', {
                    history: null, // Not updating history settings
                    settings
                });
            } catch (error) {
                console.error('Failed to update cloud settings:', error);
            }

            return true;
        }
        return false;
    },

    async logout(): Promise<void> {
        authState.set({ isLoggedIn: false, email: null });

        // Disable cloud sync in backend
        try {
            const settings = await invoke<any>('get_app_settings');
            settings.cloud.enabled = false;
            await invoke('update_app_settings', {
                history: null,
                settings
            });
        } catch (error) {
            console.error('Failed to update cloud settings:', error);
        }
    },

    // Sync actions
    async forceSyncNow(): Promise<void> {
        await invoke('force_sync_now');
    },

    async getSyncStatus(): Promise<SyncStatus> {
        const status = await invoke<SyncStatus>('get_sync_status');
        syncStatus.set(status);
        return status;
    },

    async clearSyncQueue(): Promise<void> {
        await invoke('clear_sync_queue');
    },

    // Cloud version actions
    async listCloudVersions(gameId: string, limit?: number): Promise<CloudVersion[]> {
        const versions = await invoke<CloudVersion[]>('list_cloud_versions', {
            gameId,
            limit
        });

        cloudVersions.update(map => {
            map.set(gameId, versions);
            return map;
        });

        return versions;
    },

    async downloadCloudVersion(gameId: string, versionId: string): Promise<string> {
        return await invoke<string>('download_cloud_version', {
            gameId,
            versionId
        });
    },

    async uploadCloudSave(gameId: string, emulatorId: string, localVersionId: string): Promise<any> {
        return await invoke('upload_cloud_save', {
            gameId,
            emulatorId,
            localVersionId
        });
    },

    // Cloud config actions
    async getCloudConfig(): Promise<any> {
        return await invoke('get_cloud_config');
    },

    async updateCloudConfig(config: any): Promise<any> {
        return await invoke('update_cloud_config', { newConfig: config });
    },

    async getCloudStatus(): Promise<any> {
        return await invoke('get_cloud_status');
    }
};

// Derived stores
export const isSyncing = derived(syncStatus, $status => $status.is_syncing);
export const isLoggedIn = derived(authState, $auth => $auth.isLoggedIn);
export const userEmail = derived(authState, $auth => $auth.email);
