<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import AppHeader from "../layout/AppHeader.svelte";
  import { settingsStore } from "$lib/settingsStore";

  function goBack() {
    goto("/settings", { keepFocus: true, noScroll: true });
  }

  onMount(async () => {
    await settingsStore.load();
  });

  async function handleClearCache() {
    if (confirm("Clear all history cache? This cannot be undone.")) {
      await settingsStore.clearHistory();
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
  }
</script>

<section class="settings-page">
  <div class="content-surface">
    <main class="content-body">
      <div class="header-wrapper">
        <AppHeader
          title="Storage"
          showBack
          onBack={goBack}
          onMenu={() => {}}
          sticky={false}
        />
      </div>

      <div class="content">
        {#if $settingsStore.storageInfo}
          <div class="settings-card">
            <h3>Cache Management</h3>
            <p class="description">Manage storage used by save file history</p>

            <div class="storage-stats">
              <div class="stat">
                <span class="stat-label">Total Versions</span>
                <span class="stat-value"
                  >{$settingsStore.storageInfo.total_versions}</span
                >
              </div>
              <div class="stat">
                <span class="stat-label">Cache Size</span>
                <span class="stat-value"
                  >{formatBytes(
                    $settingsStore.storageInfo.total_size_bytes
                  )}</span
                >
              </div>
            </div>

            <button class="danger-button" on:click={handleClearCache}>
              Clear All History Cache
            </button>
          </div>
        {:else}
          <div class="settings-card">
            <p class="description">Loading storage information...</p>
          </div>
        {/if}
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

  .settings-card h3 {
    margin-top: 0;
    margin-bottom: 16px;
    font-size: 1.4em;
    color: var(--text-strong);
  }

  .settings-card .description {
    font-size: 0.95em;
    color: var(--text-light);
    margin-bottom: 24px;
  }

  .storage-stats {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 24px;
  }

  .storage-stats .stat {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
  }

  .storage-stats .stat:last-child {
    border-bottom: none;
  }

  .storage-stats .stat-label {
    font-weight: 500;
    color: var(--text);
  }

  .storage-stats .stat-value {
    font-weight: 600;
    color: var(--text-strong);
  }

  .danger-button {
    background-color: var(--danger);
    color: white;
    border: none;
    padding: 12px 20px;
    border-radius: 8px;
    font-size: 1em;
    cursor: pointer;
    transition: background-color 0.2s ease;
    width: 100%;
    text-align: center;
  }

  .danger-button:hover {
    background-color: var(--danger-dark);
  }
</style>
