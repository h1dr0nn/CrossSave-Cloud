<script lang="ts">
  import { onMount } from "svelte";
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
</script>

<section class="panel">
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
</section>

<style>
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
    display: flex;
    flex-wrap: nowrap;
    white-space: nowrap;
    border-radius: 14px;
    padding: 4px;
    background: color-mix(in srgb, var(--surface-muted) 85%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 85%, transparent);
    gap: 6px;
    overflow: hidden;
  }

  .segmented button {
    border: none;
    border-radius: 12px;
    padding: 10px 14px;
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
    .segmented {
      overflow-x: auto;
      scrollbar-width: none;
    }

    .segmented::-webkit-scrollbar {
      display: none;
    }
  }
</style>
