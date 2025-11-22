import { writable } from 'svelte/store';
import type { EmulatorProfile } from './api';
import type { GameEntry } from './uiTypes';

export const profilesStore = writable<EmulatorProfile[]>([]);
export const gamesCacheStore = writable<Map<string, GameEntry[]>>(new Map());
export const selectedEmulatorIdStore = writable<string | null>(null);
