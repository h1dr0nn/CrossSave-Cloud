import { derived, get, writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

interface AuthState {
    isLoggedIn: boolean;
    email: string | null;
    token: string | null;
    deviceId: string | null;
}

export interface SyncStatus {
    queue_length: number;
    active_job: any | null;
    last_sync: string | null;
    is_syncing: boolean;
}

export interface CloudVersion {
    game_id: string;
    emulator_id: string;
    version_id: string;
    timestamp: number;
    size_bytes: number;
    hash: string;
    device_id: string;
    total_size_bytes?: number;
    file_list?: string[];
}

export interface CloudDevice {
    device_id: string;
    name: string;
    last_sync: number;
}

export type CloudMode = 'official' | 'self_host' | 'off';
export type CloudAuthMode = 'NONE' | 'ACCESS_KEY' | 'USERPASS';

export interface CloudConfig {
    mode?: CloudMode;
    base_url?: string;
    api_key?: string;
    access_key?: string;
    auth_mode?: CloudAuthMode;
    enabled?: boolean;
    timeout_seconds?: number;
    device_id?: string;
    [key: string]: unknown;
}

export interface CloudValidationResult {
    status: 'idle' | 'valid' | 'invalid';
    message: string;
}

interface LoginResult {
    token: string;
    device_id: string;
}

type DownloadPhase = 'idle' | 'downloading' | 'completed' | 'error';

interface DownloadState {
    versionId: string | null;
    progress: number;
    status: DownloadPhase;
    path: string | null;
    error: string | null;
}

const authState = writable<AuthState>({
    isLoggedIn: false,
    email: null,
    token: null,
    deviceId: null
});

const syncStatus = writable<SyncStatus>({
    queue_length: 0,
    active_job: null,
    last_sync: null,
    is_syncing: false
});

const cloudVersions = writable<Map<string, CloudVersion[]>>(new Map());
const downloadState = writable<DownloadState>({
    versionId: null,
    progress: 0,
    status: 'idle',
    path: null,
    error: null
});
const onlineStatus = writable<'online' | 'offline'>('online');
const devices = writable<CloudDevice[]>([]);
const cloudMode = writable<CloudMode>('off');
const cloudConfig = writable<CloudConfig | null>(null);
const validationResult = writable<CloudValidationResult>({ status: 'idle', message: '' });

const listeners: Promise<UnlistenFn>[] = [];

function bindEvents() {
    if (typeof window === 'undefined') return;
    if (listeners.length > 0) return;

    listeners.push(
        listen<SyncStatus>('sync://status', (event) => {
            syncStatus.set(event.payload);
        }),
        listen<{ version_id: string; progress: number }>(
            'sync://download-progress',
            (event) => {
                const payload = event.payload;
                downloadState.set({
                    versionId: payload.version_id,
                    progress: payload.progress,
                    status: 'downloading',
                    path: null,
                    error: null
                });
            }
        ),
        listen<{ version_id: string; path: string }>('sync://download-complete', (event) => {
            const payload = event.payload;
            downloadState.set({
                versionId: payload.version_id,
                progress: 100,
                status: 'completed',
                path: payload.path,
                error: null
            });
        }),
        listen<{ version_id: string; message: string }>('sync://download-error', (event) => {
            const payload = event.payload;
            downloadState.set({
                versionId: payload.version_id,
                progress: 0,
                status: 'error',
                path: null,
                error: payload.message
            });
        }),
        listen('sync://online', () => onlineStatus.set('online')),
        listen('sync://offline', () => onlineStatus.set('offline')),
        listen<{ mode: CloudMode; config?: CloudConfig }>('cloud://mode-changed', (event) => {
            cloudMode.set(event.payload.mode);
            if (event.payload.config) {
                cloudConfig.set(event.payload.config);
            }
        }),
        listen<{ config: CloudConfig }>('cloud://backend-switched', (event) => {
            const config = event.payload.config;
            cloudConfig.set(config);
            if (config?.mode) {
                cloudMode.set(config.mode);
            }
        })
    );
}

async function persistEmail(email: string) {
    if (typeof window === 'undefined') return;
    localStorage.setItem('cloud:lastEmail', email);
}

function loadPersistedEmail(): string | null {
    if (typeof window === 'undefined') return null;
    return localStorage.getItem('cloud:lastEmail');
}

export const cloudStore = {
    auth: { subscribe: authState.subscribe },
    syncStatus: { subscribe: syncStatus.subscribe },
    cloudVersions: { subscribe: cloudVersions.subscribe },
    downloadState: { subscribe: downloadState.subscribe },
    onlineStatus: { subscribe: onlineStatus.subscribe },
    devices: { subscribe: devices.subscribe },
    cloudMode: { subscribe: cloudMode.subscribe },
    cloudConfig: { subscribe: cloudConfig.subscribe },
    validation: { subscribe: validationResult.subscribe },

    async initialize(): Promise<void> {
        bindEvents();
        try {
            const config = await invoke<CloudConfig>('get_cloud_config');
            cloudConfig.set(config);
            if (config?.mode) {
                cloudMode.set(config.mode);
            }
            if (config?.enabled && config?.api_key) {
                authState.set({
                    isLoggedIn: true,
                    email: loadPersistedEmail(),
                    token: config.api_key,
                    deviceId: config.device_id ?? null
                });
            }
        } catch (error) {
            console.error('Failed to hydrate cloud auth state', error);
        }
    },

    async login(email: string, password: string): Promise<{ success: boolean; error?: string }> {
        bindEvents();
        try {
            const result = await invoke<LoginResult>('login_cloud', { email, password });
            await persistEmail(email);
            authState.set({
                isLoggedIn: true,
                email,
                token: result.token,
                deviceId: result.device_id
            });
            await this.listDevices();
            return { success: true };
        } catch (error: unknown) {
            const message = typeof error === 'string' ? error : (error as Error)?.message ?? 'Login failed';
            authState.set({ isLoggedIn: false, email: null, token: null, deviceId: null });
            return { success: false, error: message };
        }
    },

    async logout(): Promise<void> {
        try {
            await invoke('logout_cloud');
        } catch (error) {
            console.error('Failed to logout from cloud', error);
        }
        authState.set({ isLoggedIn: false, email: null, token: null, deviceId: null });
    },

    async listDevices(): Promise<CloudDevice[]> {
        bindEvents();
        const result = await invoke<CloudDevice[]>('list_devices');
        devices.set(result);
        return result;
    },

    async removeDevice(deviceId: string): Promise<void> {
        await invoke('remove_device', { device_id: deviceId });
        await this.listDevices();
    },

    async forceSyncNow(): Promise<void> {
        await invoke('force_sync_now');
    },

    async getSyncStatus(): Promise<SyncStatus> {
        bindEvents();
        const status = await invoke<SyncStatus>('get_sync_status');
        syncStatus.set(status);
        return status;
    },

    async clearSyncQueue(): Promise<void> {
        await invoke('clear_sync_queue');
    },

    async listCloudVersions(gameId: string, limit?: number): Promise<CloudVersion[]> {
        bindEvents();
        const versions = await invoke<CloudVersion[]>('list_cloud_versions', {
            gameId,
            limit
        });

        cloudVersions.update((map) => {
            map.set(gameId, versions);
            return map;
        });

        return versions;
    },

    async downloadCloudVersion(gameId: string, versionId: string): Promise<string> {
        bindEvents();
        downloadState.set({
            versionId,
            progress: 0,
            status: 'downloading',
            path: null,
            error: null
        });
        try {
            return await invoke<string>('download_cloud_version', {
                gameId,
                versionId
            });
        } catch (error: unknown) {
            const message = typeof error === 'string' ? error : (error as Error)?.message ?? 'Download failed';
            downloadState.set({
                versionId,
                progress: 0,
                status: 'error',
                path: null,
                error: message
            });
            throw message;
        }
    },

    async uploadCloudSave(gameId: string, emulatorId: string, localVersionId: string): Promise<any> {
        bindEvents();
        return await invoke('upload_cloud_save', {
            gameId,
            emulatorId,
            localVersionId
        });
    },

    async getCloudConfig(): Promise<CloudConfig | null> {
        const config = await invoke<CloudConfig>('get_cloud_config');
        cloudConfig.set(config);
        if (config?.mode) {
            cloudMode.set(config.mode);
        }
        return config;
    },

    async updateCloudConfig(config: CloudConfig): Promise<CloudConfig> {
        const updated = await invoke<CloudConfig>('update_cloud_config', { newConfig: config });
        cloudConfig.set(updated ?? config);
        return updated ?? config;
    },

    async updateCloudMode(mode: CloudMode): Promise<void> {
        bindEvents();
        await invoke('update_cloud_mode', { mode });
        cloudMode.set(mode);
    },

    async updateCloudSettings(partialConfig: Partial<CloudConfig>): Promise<CloudConfig> {
        bindEvents();
        const current = get(cloudConfig) ?? { mode: get(cloudMode) };
        const merged = { ...current, ...partialConfig, mode: partialConfig.mode ?? current.mode ?? get(cloudMode) };
        cloudConfig.set(merged);
        return await this.updateCloudConfig(merged);
    },

    async validateSelfHostSettings(config?: CloudConfig): Promise<CloudValidationResult> {
        bindEvents();
        const payload = config ?? get(cloudConfig);
        const defaultResult: CloudValidationResult = { status: 'invalid', message: 'Missing configuration' };
        if (!payload) {
            validationResult.set(defaultResult);
            return defaultResult;
        }

        try {
            const result = await invoke<{ valid: boolean; message?: string }>('validate_self_host_settings', { config: payload });
            const validation: CloudValidationResult = {
                status: result?.valid ? 'valid' : 'invalid',
                message: result?.message ?? (result?.valid ? 'Settings are valid' : 'Validation failed')
            };
            validationResult.set(validation);
            return validation;
        } catch (error: unknown) {
            const message = typeof error === 'string' ? error : (error as Error)?.message ?? 'Validation failed';
            const validation: CloudValidationResult = { status: 'invalid', message };
            validationResult.set(validation);
            return validation;
        }
    },

    async validateOfficialCloudSettings(config?: CloudConfig): Promise<CloudValidationResult> {
        bindEvents();
        const payload = config ?? get(cloudConfig);
        const defaultResult: CloudValidationResult = { status: 'invalid', message: 'Missing configuration' };
        if (!payload) {
            validationResult.set(defaultResult);
            return defaultResult;
        }

        try {
            const result = await invoke<{ valid: boolean; message?: string }>('validate_official_cloud_settings', { config: payload });
            const validation: CloudValidationResult = {
                status: result?.valid ? 'valid' : 'invalid',
                message: result?.message ?? (result?.valid ? 'Settings are valid' : 'Validation failed')
            };
            validationResult.set(validation);
            return validation;
        } catch (error: unknown) {
            const message = typeof error === 'string' ? error : (error as Error)?.message ?? 'Validation failed';
            const validation: CloudValidationResult = { status: 'invalid', message };
            validationResult.set(validation);
            return validation;
        }
    },

    async getCloudStatus(): Promise<any> {
        const status = await invoke<any>('get_cloud_status');
        onlineStatus.set(status.connected ? 'online' : 'offline');
        if (status.connected && status.device_id) {
            authState.update((state) => ({
                ...state,
                isLoggedIn: true,
                deviceId: status.device_id,
                email: state.email ?? loadPersistedEmail(),
                token: state.token
            }));
        }
        return status;
    }
};

export const isSyncing = derived(syncStatus, ($status) => $status.is_syncing);
export const isLoggedIn = derived(authState, ($auth) => $auth.isLoggedIn);
export const userEmail = derived(authState, ($auth) => $auth.email);
export const online = derived(onlineStatus, ($status) => $status === 'online');
