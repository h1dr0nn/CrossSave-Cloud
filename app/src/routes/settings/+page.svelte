<script lang="ts">
  import { onMount } from "svelte";

  import { theme } from "$lib/themeStore";

  const themeStore = theme;
  let hasManualPreference = false;

  onMount(() => {
    hasManualPreference = typeof localStorage !== "undefined" && !!localStorage.getItem("ui-theme");
  });

  function setLight() {
    hasManualPreference = true;
    themeStore.setTheme("light");
  }

  function setDark() {
    hasManualPreference = true;
    themeStore.setTheme("dark");
  }

  function useSystem() {
    hasManualPreference = false;
    themeStore.useSystemTheme();
  }
</script>

<section class="settings">
  <div class="header">
    <div>
      <p class="eyebrow">Preferences</p>
      <h1>Settings</h1>
      <p class="subtitle">Control theme and accessibility for CrossSave Cloud.</p>
    </div>
  </div>

  <div class="panel">
    <div>
      <p class="label">Appearance</p>
      <p class="helper">Pick a light or dark look, or follow your operating system.</p>
    </div>
    <div class="actions">
      <button class="pill" on:click={setLight} aria-pressed={$themeStore === "light"}>
        <span>Light</span>
      </button>
      <button class="pill" on:click={setDark} aria-pressed={$themeStore === "dark"}>
        <span>Dark</span>
      </button>
      <button class="pill" on:click={useSystem} aria-pressed={!hasManualPreference}>
        System
      </button>
    </div>
    <div class="hint">
      <span class="dot" aria-hidden="true"></span>
      <p>Current theme: {$themeStore}</p>
    </div>
  </div>
</section>

<style>
  .settings {
    padding: clamp(16px, 3vw, 32px);
    display: grid;
    gap: clamp(14px, 2vw, 18px);
    background: var(--bg);
    color: var(--text);
    min-height: 100vh;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  h1 {
    margin: 4px 0 6px;
    font-size: clamp(1.6rem, 1vw + 1.2rem, 2rem);
  }

  .eyebrow {
    margin: 0;
    font-size: clamp(0.85rem, 0.4vw + 0.75rem, 1rem);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .subtitle {
    margin: 0;
    color: var(--muted);
  }

  .panel {
    border: 1px solid var(--border);
    background: var(--surface);
    border-radius: clamp(12px, 1vw, 16px);
    padding: clamp(14px, 2vw, 20px);
    box-shadow: var(--shadow-strong);
    display: grid;
    gap: clamp(12px, 1vw, 16px);
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
    .header {
      flex-direction: column;
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
