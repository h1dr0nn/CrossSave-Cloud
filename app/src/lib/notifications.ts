import { writable } from "svelte/store";

type NotificationLevel = "info" | "error";

export interface NotificationEntry {
  id: number;
  level: NotificationLevel;
  message: string;
  timestamp: number;
}

let counter = 0;

export const notifications = writable<NotificationEntry[]>([]);

export function pushNotification(level: NotificationLevel, message: string) {
  const entry: NotificationEntry = {
    id: ++counter,
    level,
    message,
    timestamp: Date.now()
  };

  notifications.update((current) => [entry, ...current].slice(0, 100));
}

export function pushError(message: string) {
  pushNotification("error", message);
}

export function pushInfo(message: string) {
  pushNotification("info", message);
}

export function dismissNotification(id: number) {
  notifications.update((current) => current.filter((entry) => entry.id !== id));
}
