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
    </button>
    <h1>Settings</h1>
    <span class="spacer" aria-hidden="true"></span>
  </header>

  <div class="panel">
    <div class="section-header">
      <p class="label">Theme</p>
      <span class="hint">Applies instantly</span>
    </div>
    <div class="segmented" role="group" aria-label="Theme selector">
      <button class:active={$themePreference === "light"} aria-pressed={$themePreference === "light"} on:click={setLight}>
        Light
      </button>
      <button class:active={$themePreference === "dark"} aria-pressed={$themePreference === "dark"} on:click={setDark}>
        Dark
      </button>
      <button class:active={$themePreference === "system"} aria-pressed={$themePreference === "system"} on:click={useSystem}>
        System
      </button>
    </div>
    <div class="status">
      <span class="dot" aria-hidden="true"></span>
      <p>Current theme: {$activeTheme}{hasManualPreference ? " (manual)" : " (system)"}</p>
    </div>
  </div>
</section>

<style>
  .settings {
    padding: clamp(16px, 3vw, 32px);
    display: flex;
    flex-direction: column;
    gap: 18px;
    background: var(--bg);
    color: var(--text);
    min-height: 100vh;
  }

  .settings-bar {
    position: sticky;
    top: 0;
    z-index: 8;
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 10px;
    padding: 12px 14px;
    border-radius: 18px;
    background: color-mix(in srgb, var(--surface) 94%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 90%, transparent);
    box-shadow: var(--shadow-soft);
    backdrop-filter: blur(14px);
  }

  .icon-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 36px;
    width: 36px;
    padding: 0;
    border-radius: 12px;
    border: 1px solid color-mix(in srgb, var(--border) 90%, transparent);
    background: linear-gradient(120deg, color-mix(in srgb, var(--surface) 94%, transparent), var(--surface));
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

  h1 {
    text-align: center;
    margin: 0;
    font-size: clamp(1.3rem, 0.7vw + 1.15rem, 1.7rem);
    letter-spacing: -0.01em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .spacer {
    width: 36px;
    height: 36px;
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
    backdrop-filter: blur(12px);
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    flex-wrap: wrap;
  }

  .label {
    margin: 0;
    font-weight: 700;
  }

  .hint {
    color: var(--muted);
    font-size: 0.95rem;
  }

  .segmented {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    border-radius: 14px;
    padding: 4px;
    background: color-mix(in srgb, var(--surface-muted) 85%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 85%, transparent);
    gap: 6px;
  }

  .segmented button {
    border: none;
    border-radius: 12px;
    padding: 10px;
    background: transparent;
    color: var(--text);
    font-weight: 700;
    cursor: pointer;
    transition: background 0.2s ease, box-shadow 0.2s ease, transform 0.1s ease;
  }

  .segmented button.active,
  .segmented button[aria-pressed="true"] {
    background: linear-gradient(135deg, color-mix(in srgb, var(--accent-muted) 80%, var(--surface)), var(--surface));
    box-shadow: 0 10px 18px color-mix(in srgb, var(--accent-strong) 18%, transparent);
    transform: translateY(-1px);
    border: 1px solid color-mix(in srgb, var(--accent) 40%, var(--border));
  }

  .status {
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
    box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent-muted) 40%, transparent);
  }

  @media (max-width: 640px) {
    h1 {
      font-size: 1.3rem;
    }

    .segmented {
      grid-template-columns: 1fr;
    }
  }
</style>
