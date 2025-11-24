<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import AppHeader from "../layout/AppHeader.svelte";
  import { settingsStore } from "$lib/settingsStore";
  import { openFolder } from "$lib/api";

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

  async function handleOpenFolder() {
    if ($settingsStore.storageInfo?.history_path) {
      await openFolder($settingsStore.storageInfo.history_path);
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

      <div class="settings-container">
        {#if $settingsStore.storageInfo}
          <div class="section-group">
            <div class="section-header">
              <p class="section-title">Cache Management</p>
            </div>
            <div class="settings-card">
              <div class="card-content">
                <p class="description">
                  Manage storage used by save file history.
                </p>

                <div class="input-stack">
                  <div class="setting-item">
                    <div class="folder-row">
                      <span class="label">History Folder</span>
                      <div class="path-group">
                        <code class="path"
                          >{$settingsStore.storageInfo.history_path}</code
                        >
                        <button
                          class="icon-button"
                          on:click={handleOpenFolder}
                          title="Open Folder"
                        >
                          <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="16"
                            height="16"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                          >
                            <path
                              d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"
                            ></path>
                            <polyline points="15 3 21 3 21 9"></polyline>
                            <line x1="10" y1="14" x2="21" y2="3"></line>
                          </svg>
                        </button>
                      </div>
                    </div>
                  </div>

                  <div class="divider"></div>

                  <div class="setting-item">
                    <div class="setting-label-row">
                      <span class="label">Total Versions</span>
                      <span class="value-badge"
                        >{$settingsStore.storageInfo.total_versions}</span
                      >
                    </div>
                  </div>

                  <div class="divider"></div>

                  <div class="setting-item">
                    <div class="setting-label-row">
                      <span class="label">Current Size</span>
                      <span class="value-badge"
                        >{formatBytes(
                          $settingsStore.storageInfo.total_size_bytes,
                        )}</span
                      >
                    </div>
                  </div>
                </div>

                <div class="actions-row">
                  <button
                    class="btn-danger btn-full"
                    on:click={handleClearCache}
                  >
                    Clear All History Cache
                  </button>
                </div>
              </div>
            </div>
          </div>
        {:else}
          <div class="settings-card">
            <div class="card-content">
              <p class="description">Loading storage information...</p>
            </div>
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

  .settings-container {
    width: 100%;
    display: grid;
    gap: 32px;
    align-content: start;
  }

  .section-group {
    display: grid;
    gap: 8px;
  }

  .section-header {
    padding: 0 16px;
  }

  .section-title {
    margin: 0;
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
  }

  .settings-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
    box-shadow: var(--shadow-soft);
  }

  .card-content {
    padding: 24px;
  }

  .description {
    margin: 0 0 24px 0;
    color: var(--text-secondary);
    font-size: 0.95rem;
    line-height: 1.5;
  }

  /* Input Stack Styling */
  .input-stack {
    display: grid;
    gap: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-secondary);
    overflow: hidden;
    margin-bottom: 24px;
  }

  .setting-item {
    padding: 16px;
    background: var(--surface);
    min-width: 0; /* Fix grid item overflow */
  }

  .divider {
    height: 1px;
    background: var(--border);
    margin: 0 16px;
  }

  .setting-label-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .folder-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    min-width: 0; /* Fix flex item overflow */
  }

  .path-group {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
    justify-content: flex-end;
  }

  .label {
    font-weight: 500;
    color: var(--text-primary);
    font-size: 0.95rem;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .value-badge {
    padding: 4px 8px;
    background: var(--bg-secondary);
    border-radius: 4px;
    font-size: 0.85rem;
    color: var(--text-secondary);
    font-family: "SF Mono", "Monaco", monospace;
    border: 1px solid var(--border);
  }

  .path {
    font-family: "SF Mono", "Monaco", monospace;
    background: var(--bg-secondary);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.85rem;
    color: var(--text-secondary);
    display: block;
    border: 1px solid var(--border);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
    flex: 1; /* Allow growing and shrinking */
    text-align: right; /* Align text to right if truncated from left? No, ellipsis is usually at end */
  }

  .icon-button {
    background: none;
    border: none;
    padding: 4px;
    cursor: pointer;
    color: var(--text-secondary);
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
    border: 1px solid transparent;
  }

  .icon-button:hover {
    background: var(--bg-hover);
    color: var(--text);
    border-color: var(--border);
  }

  /* Actions */
  .actions-row {
    display: flex;
    justify-content: center;
  }

  .btn-danger {
    background: color-mix(in srgb, var(--danger) 10%, transparent);
    color: var(--danger);
    border: 1px solid color-mix(in srgb, var(--danger) 20%, transparent);
    padding: 0.75rem 1.5rem;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 1rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-full {
    width: 100%;
  }

  .btn-danger:hover {
    background: var(--danger);
    color: white;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px color-mix(in srgb, var(--danger) 25%, transparent);
  }
</style>
