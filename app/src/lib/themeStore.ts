import { writable } from "svelte/store";

type Theme = "light" | "dark";
type ThemePreference = Theme | "system";

const STORAGE_KEY = "ui-theme";
const isBrowser = typeof window !== "undefined";

function getSystemTheme(): Theme {
  if (!isBrowser) return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function getInitialPreference(): ThemePreference {
  if (!isBrowser) return "light";
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "light" || stored === "dark" || stored === "system") {
    return stored;
  }
  return "system";
}

function createThemeStore() {
  const initialPreference = getInitialPreference();
  const initialTheme = initialPreference === "system" ? getSystemTheme() : initialPreference;

  const preferenceStore = writable<ThemePreference>(initialPreference);
  const activeThemeStore = writable<Theme>(initialTheme);

  function applyTheme(preference: ThemePreference) {
    if (!isBrowser) return;
    const theme = preference === "system" ? getSystemTheme() : preference;
    document.body.dataset.theme = theme;
    document.documentElement.style.colorScheme = theme;
    activeThemeStore.set(theme);
  }

  applyTheme(initialPreference);
  const mediaQuery = isBrowser ? window.matchMedia("(prefers-color-scheme: dark)") : null;

  const handleSystemChange = (event: MediaQueryListEvent) => {
    preferenceStore.update((current) => {
      if (current !== "system") return current;
      applyTheme(event.matches ? "dark" : "light");
      return current;
    });
  };

  if (mediaQuery) {
    mediaQuery.addEventListener("change", handleSystemChange);
  }

  preferenceStore.subscribe((value) => {
    if (isBrowser) {
      localStorage.setItem(STORAGE_KEY, value);
    }
    applyTheme(value);
  });

  function setTheme(theme: ThemePreference) {
    preferenceStore.set(theme);
  }

  function toggleTheme() {
    preferenceStore.update((current) => {
      const next = current === "dark" ? "light" : "dark";
      return next;
    });
  }

  function useSystemTheme() {
    preferenceStore.set("system");
  }

  return {
    subscribe: preferenceStore.subscribe,
    setTheme,
    toggleTheme,
    useSystemTheme,
    preference: preferenceStore,
    activeTheme: activeThemeStore
  };
}

export const theme = createThemeStore();
