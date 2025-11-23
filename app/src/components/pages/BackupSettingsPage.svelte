<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import AppHeader from "../layout/AppHeader.svelte";
  import { settingsStore } from "$lib/settingsStore";

  function goBack() {
    goto("/settings", { keepFocus: true, noScroll: true });
  }

  let retentionLimit = 10;
  let autoDelete = true;

  onMount(async () => {
    await settingsStore.load();
  });

  $: if ($settingsStore.appSettings) {
    retentionLimit = $settingsStore.appSettings.retention_limit;
    autoDelete = $settingsStore.appSettings.auto_delete_old_versions;
  }

  async function handleSave() {
    if ($settingsStore.appSettings) {
      await settingsStore.applySettings({
        ...$settingsStore.appSettings,
        retention_limit: retentionLimit,
        auto_delete_old_versions: autoDelete,
      });
    }
  }
</script>

<section class="settings-page">
  <div class="content-surface">
    <main class="content-body">
      <div class="header-wrapper">
        <AppHeader
          title="Backup & Retention"
          showBack
          onBack={goBack}
          onMenu={() => {}}
          sticky={false}
        />
      </div>

      <div class="content">
        <div class="settings-card">
          <h3>Retention Policy</h3>
          <p class="description">
            Configure how many save file versions to keep per game.
          </p>

          <div class="setting-group">
            <label for="retention">
              <span>Retention Limit</span>
              <span class="value">{retentionLimit} versions</span>
            </label>
            <input
              id="retention"
              type="range"
              min={$settingsStore.retentionBounds[0]}
              max={$settingsStore.retentionBounds[1]}
              bind:value={retentionLimit}
            />
          </div>

          <div class="setting-group">
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={autoDelete} />
              <span>Auto-delete old versions</span>
            </label>
            <p class="hint">
              Automatically remove versions beyond the retention limit
            </p>
          </div>

          <button
            class="save-button"
            on:click={handleSave}
            disabled={$settingsStore.savingSettings}
          >
            {$settingsStore.savingSettings ? "Saving..." : "Save Changes"}
          </button>
        </div>
      </div>
    </main>
  </div>
</section>

<style>
  .settings-page {
    display: grid;
    grid-template-columns: 1fr;
    min-height: 100vh;
    width: 100%;
    background: var(--bg);
    color: var(--text);
  }

  .content-surface {
    display: grid;
    grid-template-rows: 1fr;
    max-width: 1360px;
    margin: 0 auto;
    width: 100%;
    min-height: 100vh;
  }

  .content-body {
    overflow-y: auto;
    overflow-x: hidden;
    scroll-behavior: smooth;
    padding: clamp(16px, 3vw, 32px);
    padding-top: max(clamp(16px, 3vw, 32px), env(safe-area-inset-top));
    padding-bottom: max(clamp(16px, 3vw, 32px), env(safe-area-inset-bottom));
    padding-left: max(clamp(16px, 3vw, 32px), env(safe-area-inset-left));
    padding-right: max(clamp(16px, 3vw, 32px), env(safe-area-inset-right));
  }

  .header-wrapper {
    margin-bottom: clamp(16px, 3vw, 32px);
  }

  .settings-card {
    background: var(--surface);
    border: 1px solid color-mix(in srgb, var(--border) 90%, transparent);
    border-radius: 12px;
    padding: 24px;
    margin-bottom: 24px;
    box-shadow: var(--shadow-soft);
  }

  h3 {
    margin: 0 0 8px 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
  }

  .description {
    margin: 0 0 24px 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .setting-group {
    margin-bottom: 24px;
  }

  .setting-group label {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
    font-weight: 500;
    color: var(--text);
  }

  .value {
    color: var(--muted);
    font-size: 0.9rem;
  }

  input[type="range"] {
    width: 100%;
    height: 6px;
    border-radius: 3px;
    background: var(--surface-muted);
    outline: none;
    -webkit-appearance: none;
  }

  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--accent);
    cursor: pointer;
  }

  .checkbox-label {
    display: flex !important;
    gap: 12px;
    cursor: pointer;
  }

  input[type="checkbox"] {
    width: 20px;
    height: 20px;
    cursor: pointer;
  }

  .hint {
    margin: 8px 0 0 32px;
    font-size: 0.85rem;
    color: var(--muted);
  }

  .save-button {
    width: 100%;
    padding: 12px;
    border: none;
    border-radius: 8px;
    background: var(--accent);
    color: white;
    font-weight: 500;
    font-size: 1rem;
    cursor: pointer;
    transition: opacity 0.2s;
  }

  .save-button:hover:not(:disabled) {
    opacity: 0.9;
  }

  .save-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
