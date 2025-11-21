<script lang="ts">
import { onMount } from "svelte";
import { goto } from "$app/navigation";

import { theme } from "$lib/themeStore";

const themePreference = theme.preference;
const activeTheme = theme.activeTheme;
let hasManualPreference = false;

onMount(() => {
  if (typeof localStorage !== "undefined") {
    const stored = localStorage.getItem("ui-theme");
    hasManualPreference = stored === "light" || stored === "dark";
  }
});

function setLight() {
  hasManualPreference = true;
  theme.setTheme("light");
}

function setDark() {
  hasManualPreference = true;
  theme.setTheme("dark");
}

function useSystem() {
  hasManualPreference = false;
  theme.setTheme("system");
}

function goBack() {
  goto("/", { keepFocus: true, noScroll: true });
}
</script>

<section class="settings">
  <header class="settings-bar">
    <button class="icon-button" on:click={goBack} aria-label="Back to dashboard">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M14.5 6 8.5 12l6 6" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
      </svg>
      <span>Back</span>
    </button>
    <div class="headline">
      <p class="eyebrow">Preferences</p>
      <h1>Settings</h1>
      <p class="subtitle">Control theme and accessibility for CrossSave Cloud.</p>
    </div>
  </header>

  <div class="panel">
    <div>
      <p class="label">Appearance</p>
      <p class="helper">Pick a light or dark look, or follow your operating system.</p>
    </div>
    <div class="actions">
      <button class="pill" on:click={setLight} aria-pressed={$themePreference === "light"}>
        <span>Light</span>
      </button>
      <button class="pill" on:click={setDark} aria-pressed={$themePreference === "dark"}>
        <span>Dark</span>
      </button>
      <button class="pill" on:click={useSystem} aria-pressed={$themePreference === "system"}>
        System
      </button>
    </div>
    <div class="hint">
      <span class="dot" aria-hidden="true"></span>
      <p>Current theme: {$activeTheme}</p>
    </div>
  </div>
</section>

<style>
  .settings {
    padding: clamp(16px, 3vw, 28px);
    display: flex;
    flex-direction: column;
    gap: 16px;
    background: var(--bg);
    color: var(--text);
    min-height: 100vh;
  }

  .settings-bar {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 12px;
    align-items: center;
    padding: 14px 16px;
    border-radius: var(--radius);
    background: color-mix(in srgb, var(--surface) 94%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 88%, transparent);
    box-shadow: var(--shadow-soft);
  }

  .icon-button {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--surface);
    cursor: pointer;
    color: var(--text);
    font-weight: 600;
    transition: transform 0.15s ease, box-shadow 0.2s ease, border-color 0.2s ease;
  }

  .icon-button:hover {
    transform: translateY(-1px);
    border-color: var(--accent);
    box-shadow: var(--shadow);
  }

  .icon-button svg {
    width: 20px;
    height: 20px;
  }

  .headline h1 {
    margin: 6px 0 6px;
    font-size: clamp(1.4rem, 0.8vw + 1.2rem, 1.8rem);
  }

  .eyebrow {
    margin: 0;
    font-size: 0.85rem;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .subtitle {
    margin: 0;
    color: var(--muted);
  }

  .panel {
    border: 1px solid color-mix(in srgb, var(--border) 88%, transparent);
    background: linear-gradient(180deg, color-mix(in srgb, var(--surface) 94%, transparent), var(--surface));
    border-radius: var(--radius);
    padding: 16px;
    box-shadow: var(--shadow-soft);
    display: grid;
    gap: 14px;
    align-content: start;
  }

  .label {
    margin: 0 0 4px;
    font-weight: 700;
  }

  .helper {
    margin: 0;
    color: var(--muted);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
  }

  .pill {
    padding: 10px 14px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--surface-muted);
    color: var(--text);
    cursor: pointer;
    font-weight: 700;
    transition: border-color 0.2s ease, transform 0.15s ease, box-shadow 0.2s ease;
  }

  .pill[aria-pressed="true"] {
    border-color: var(--accent);
    box-shadow: 0 10px 18px color-mix(in srgb, var(--accent-strong) 20%, transparent);
    transform: translateY(-1px);
  }

  .pill:hover {
    border-color: var(--accent);
  }

  .hint {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--muted);
  }

  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--accent);
    display: inline-block;
  }

  @media (max-width: 640px) {
    .settings-bar {
      grid-template-columns: 1fr;
      align-items: flex-start;
    }

    .actions {
      width: 100%;
    }

    .pill {
      flex: 1;
      text-align: center;
    }
  }
</style>
