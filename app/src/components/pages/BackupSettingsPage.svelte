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
    autoDelete = $settingsStore.appSettings.auto_delete;
  }

  async function handleSave() {
    if ($settingsStore.appSettings) {
      await settingsStore.applySettings({
        ...$settingsStore.appSettings,
        retention_limit: retentionLimit,
        auto_delete: autoDelete,
      });
    }
  }
</script>

<section class="settings-page">
  <div class="content-surface">
    <main class="content-body">
      <div class="header-wrapper">
        <AppHeader
          title="Backup"
          showBack
          onBack={goBack}
          onMenu={() => {}}
          sticky={false}
        />
      </div>

      <div class="settings-container">
        <div class="section-group">
          <div class="section-header">
            <p class="section-title">Retention Policy</p>
          </div>
          <div class="settings-card">
            <div class="card-content">
              <p class="description">
                Configure how many save file versions to keep per game. Older
                versions will be automatically managed based on these settings.
              </p>

              <div class="input-stack">
                <div class="setting-item">
                  <div class="setting-label-row">
                    <span class="label">Retention Limit</span>
                    <span class="value-badge">{retentionLimit} versions</span>
                  </div>
                  <div class="range-wrapper">
                    <input
                      type="range"
                      min={$settingsStore.retentionBounds[0]}
                      max={$settingsStore.retentionBounds[1]}
                      bind:value={retentionLimit}
                    />
                    <div class="range-labels">
                      <span>{$settingsStore.retentionBounds[0]}</span>
                      <span>{$settingsStore.retentionBounds[1]}</span>
                    </div>
                  </div>
                </div>

                <div class="divider"></div>

                <div class="setting-item">
                  <label class="checkbox-row">
                    <div class="checkbox-text">
                      <span class="label">Auto-delete old versions</span>
                      <span class="hint"
                        >Automatically remove versions beyond the limit</span
                      >
                    </div>
                    <input type="checkbox" bind:checked={autoDelete} />
                  </label>
                </div>
              </div>

              <div class="actions-row">
                <button
                  class="btn-primary btn-full"
                  on:click={handleSave}
                  disabled={$settingsStore.savingSettings}
                >
                  {$settingsStore.savingSettings ? "Saving..." : "Save Changes"}
                </button>
              </div>
            </div>
          </div>
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

  /* Input Stack Styling similar to CloudPage */
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
    margin-bottom: 12px;
  }

  .label {
    font-weight: 500;
    color: var(--text-primary);
    font-size: 0.95rem;
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

  /* Range Slider */
  .range-wrapper {
    padding: 0 4px;
  }

  input[type="range"] {
    width: 100%;
    height: 6px;
    border-radius: 3px;
    background: var(--bg-secondary);
    outline: none;
    appearance: none;
    -webkit-appearance: none;
    cursor: pointer;
    border: 1px solid var(--border);
  }

  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--accent);
    border: 2px solid var(--surface);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    cursor: pointer;
    transition: transform 0.1s;
  }

  input[type="range"]::-webkit-slider-thumb:hover {
    transform: scale(1.1);
  }

  .range-labels {
    display: flex;
    justify-content: space-between;
    margin-top: 8px;
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  /* Checkbox Row */
  .checkbox-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
    width: 100%;
  }

  .checkbox-text {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .hint {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  input[type="checkbox"] {
    width: 22px;
    height: 22px;
    cursor: pointer;
    border-radius: 6px;
    border: 1px solid var(--border);
    appearance: none;
    -webkit-appearance: none;
    background: var(--bg-secondary);
    display: grid;
    place-content: center;
    transition: all 0.2s;
  }

  input[type="checkbox"]::before {
    content: "";
    width: 12px;
    height: 12px;
    transform: scale(0);
    transition: 0.12s transform ease-in-out;
    box-shadow: inset 1em 1em white;
    transform-origin: center;
    clip-path: polygon(14% 44%, 0 65%, 50% 100%, 100% 16%, 80% 0%, 43% 62%);
  }

  input[type="checkbox"]:checked {
    background: var(--accent);
    border-color: var(--accent);
  }

  input[type="checkbox"]:checked::before {
    transform: scale(1);
  }

  /* Actions */
  .actions-row {
    display: flex;
    justify-content: center;
  }

  .btn-primary {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 1rem;
    background: var(--accent);
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px color-mix(in srgb, var(--accent) 25%, transparent);
  }

  .btn-full {
    width: 100%;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent-strong);
    transform: translateY(-1px);
    box-shadow: 0 6px 16px color-mix(in srgb, var(--accent) 30%, transparent);
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
    box-shadow: none;
  }
</style>
