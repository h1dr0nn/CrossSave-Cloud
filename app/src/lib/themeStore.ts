import { writable } from "svelte/store";

type Theme = "light" | "dark";

const STORAGE_KEY = "ui-theme";
const isBrowser = typeof window !== "undefined";

function getSystemTheme(): Theme {
  if (!isBrowser) return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function getInitialTheme(): Theme {
  if (!isBrowser) return "light";
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "light" || stored === "dark") {
    return stored;
  }
  return getSystemTheme();
}

function applyTheme(theme: Theme) {
  if (!isBrowser) return;
  const body = document.body;
  body.classList.remove("theme-light", "theme-dark");
  body.classList.add(`theme-${theme}`);
}

function createThemeStore() {
  const initial = getInitialTheme();
  applyTheme(initial);

  const { subscribe, set, update } = writable<Theme>(initial);

  const mediaQuery = isBrowser ? window.matchMedia("(prefers-color-scheme: dark)") : null;
  const handleSystemChange = (event: MediaQueryListEvent) => {
    const stored = isBrowser ? localStorage.getItem(STORAGE_KEY) : null;
    if (stored === "light" || stored === "dark") return;
    const nextTheme: Theme = event.matches ? "dark" : "light";
    applyTheme(nextTheme);
    set(nextTheme);
  };

  if (mediaQuery) {
    mediaQuery.addEventListener("change", handleSystemChange);
  }

  subscribe((value) => applyTheme(value));

  function setTheme(theme: Theme) {
    if (isBrowser) {
      localStorage.setItem(STORAGE_KEY, theme);
    }
    applyTheme(theme);
    set(theme);
  }

  function toggleTheme() {
    update((current) => {
      const next = current === "dark" ? "light" : "dark";
      if (isBrowser) {
        localStorage.setItem(STORAGE_KEY, next);
      }
      applyTheme(next);
      return next;
    });
  }

  function useSystemTheme() {
    if (isBrowser) {
      localStorage.removeItem(STORAGE_KEY);
    }
    const next = getSystemTheme();
    applyTheme(next);
    set(next);
  }

  return { subscribe, setTheme, toggleTheme, useSystemTheme };
}

export const theme = createThemeStore();
