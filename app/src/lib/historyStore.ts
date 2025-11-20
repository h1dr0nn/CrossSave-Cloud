import { writable } from "svelte/store";
import type { HistoryEntry } from "./api";

export interface HistoryState {
  gameId: string;
  entries: HistoryEntry[];
}

export const historyState = writable<HistoryState>({ gameId: "", entries: [] });

export function setHistory(gameId: string, entries: HistoryEntry[]) {
  historyState.set({ gameId, entries });
}

export function addHistoryEntry(entry: HistoryEntry) {
  historyState.update((state) => {
    if (state.gameId && state.gameId !== entry.metadata.game_id) {
      return state;
    }

    const filtered = state.entries.filter(
      (existing) => existing.metadata.version_id !== entry.metadata.version_id
    );

    return {
      gameId: state.gameId || entry.metadata.game_id,
      entries: [entry, ...filtered].sort(
        (a, b) => b.metadata.timestamp - a.metadata.timestamp
      )
    };
  });
}
