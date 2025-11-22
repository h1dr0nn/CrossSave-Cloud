<script lang="ts">
  import { openPath } from "@tauri-apps/plugin-opener";
  import { pushInfo, pushError } from "../lib/notifications";

  export let profile: EmulatorProfile | null = null;
  export let loading = false;

  $: matchedCount = profile
    ? Math.max(
        profile.file_patterns.length *
          Math.max(profile.default_save_paths.length, 1),
        0
      )
    : 0;

  async function openFolder() {
    if (!profile?.default_save_paths.length) return;

    for (const path of profile.default_save_paths) {
      try {
        await openPath(path);
        return;
      } catch (error) {
        console.warn(`Failed to open ${path}`, error);
      }
    }

    pushInfo("Could not open specific folder. Opening file manager...");

    try {
      // Fallback to root storage
      await openPath("/storage/emulated/0");
    } catch (e) {
      console.error("Failed to open root storage", e);
      pushError("Unable to open file manager.");
    }
  }

  import { checkPathStatus, type PathStatus } from "../lib/api";

  let pathStatuses: PathStatus[] = [];
  let showDebug = false;

  async function debugPaths() {
    if (!profile) return;
    try {
      pathStatuses = await checkPathStatus(profile.emulator_id);
      showDebug = true;
    } catch (e) {
      console.error("Failed to check paths", e);
      pushError("Failed to check path status");
    }
  }
</script>

<section class="panel">
  <header>
    <div>
      <p class="eyebrow">Save folder preview</p>
      <h2>{profile?.name ?? "Select an emulator"}</h2>
    </div>
    <div class="actions">
      {#if profile}
        <button class="debug-btn" on:click={debugPaths} title="Check paths">
          <svg
            viewBox="0 0 24 24"
            aria-hidden="true"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </button>
      {/if}
      <button
        class="open"
        on:click={openFolder}
        disabled={!profile?.default_save_paths.length}
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path
            d="M4.5 6a2 2 0 0 1 2-2h2.9a1 1 0 0 1 .83.45L11.8 6H17.5A2.5 2.5 0 0 1 20 8.5v8.25a1.75 1.75 0 0 1-1.75 1.75h-11.5A2.25 2.25 0 0 1 4.5 16.25Z"
            fill="currentColor"
          />
        </svg>
        Open folder
      </button>
    </div>
  </header>

  {#if showDebug}
    <div class="debug-panel">
      <div class="debug-header">
        <h3>Path Status</h3>
        <button on:click={() => (showDebug = false)}>Close</button>
      </div>
      <ul class="path-list">
        {#each pathStatuses as status}
          <li
            class:valid={status.exists && status.is_dir}
            class:invalid={!status.exists || !status.is_dir}
          >
            <div class="status-icon">
              {#if status.exists && status.is_dir}
                <svg
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  class="icon-success"
                >
                  <polyline points="20 6 9 17 4 12" />
                </svg>
              {:else}
                <svg
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  class="icon-error"
                >
                  <circle cx="12" cy="12" r="10" />
                  <line x1="12" y1="8" x2="12" y2="12" />
                  <line x1="12" y1="16" x2="12.01" y2="16" />
                </svg>
              {/if}
            </div>
            <div class="path-details">
              <span class="path">{status.path}</span>
              <span class="error">
                {#if !status.exists}
                  Not found
                {:else if !status.is_dir}
                  Not a directory
                {:else if status.error}
                  Error: {status.error}
                {:else}
                  OK
                {/if}
              </span>
            </div>
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  {#if loading}
    <p class="empty">Loading emulator profile…</p>
  {:else if !profile}
    <p class="empty">Choose an emulator to preview save details.</p>
  {:else}
    <div class="fields">
      <div class="field">
        <p class="label">Save path</p>
        <div class="value" title={profile.default_save_paths.join(", ")}>
          {profile.default_save_paths.join("\n")}
        </div>
      </div>
      <div class="field">
        <p class="label">Pattern</p>
        <div class="value chips">
          {#each profile.file_patterns as pattern}
            <code>{pattern}</code>
          {/each}
        </div>
      </div>
      <div class="preview">
        <div class="preview-head">
          <div>
            <p class="eyebrow">Matched files</p>
            <strong>{matchedCount} detected</strong>
          </div>
          <span class="pill">Live preview</span>
        </div>
        <ul>
          {#each profile.file_patterns as pattern}
            <li>{pattern}</li>
          {/each}
        </ul>
      </div>
    </div>
  {/if}
</section>

<style>
  .panel {
    background: color-mix(in srgb, var(--surface) 94%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 88%, transparent);
    border-radius: var(--radius);
    padding: clamp(14px, 2vw, 18px);
    box-shadow: var(--shadow-soft);
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-height: 0;
    color: var(--text);
    backdrop-filter: blur(12px) saturate(1.05);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }

  h2 {
    margin: 2px 0 0;
    font-size: clamp(1.1rem, 0.7vw + 1rem, 1.35rem);
    color: var(--text);
  }

  .eyebrow {
    margin: 0;
    color: var(--muted);
    font-size: 0.8rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .open {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 12px 14px;
    border-radius: 14px;
    border: 1px solid var(--accent);
    background: linear-gradient(120deg, var(--accent-strong), var(--accent));
    color: #fff;
    font-weight: 700;
    cursor: pointer;
    transition:
      transform 0.15s ease,
      box-shadow 0.2s ease,
      opacity 0.2s ease;
    box-shadow: 0 12px 24px
      color-mix(in srgb, var(--accent-strong) 28%, transparent);
  }

  .open:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    box-shadow: none;
  }

  .open:not(:disabled):hover {
    transform: translateY(-1px);
    box-shadow: 0 12px 24px
      color-mix(in srgb, var(--accent-strong) 30%, transparent);
  }

  .open svg {
    width: 20px;
    height: 20px;
  }

  .fields {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .label {
    font-weight: 700;
    color: var(--text);
    font-size: 0.98rem;
    margin: 0;
  }

  .value {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 12px;
    background: color-mix(in srgb, var(--surface-muted) 90%, transparent);
    white-space: pre-wrap;
    color: var(--text);
    min-height: 56px;
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    word-break: break-all;
  }

  .value.chips {
    gap: 10px;
  }

  code {
    background: color-mix(in srgb, var(--card-contrast) 88%, transparent);
    color: var(--accent-strong);
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    font-size: 0.88rem;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.15);
  }

  .preview {
    border: 1px solid color-mix(in srgb, var(--border) 85%, transparent);
    border-radius: var(--radius-sm);
    padding: 12px;
    display: grid;
    grid-template-columns: 1fr;
    gap: 10px;
    background: linear-gradient(
      180deg,
      color-mix(in srgb, var(--surface-muted) 90%, transparent),
      var(--surface)
    );
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.05);
  }

  .preview-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }

  .preview strong {
    font-size: 1rem;
    color: var(--text);
  }

  .preview ul {
    margin: 0;
    padding-left: 18px;
    display: grid;
    gap: 6px;
  }

  .pill {
    padding: 6px 10px;
    border-radius: 999px;
    border: 1px solid color-mix(in srgb, var(--accent) 40%, var(--border));
    background: color-mix(in srgb, var(--accent-muted) 70%, var(--surface));
    color: var(--accent-strong);
    font-weight: 700;
    font-size: 0.85rem;
  }

  .preview li {
    color: var(--muted);
  }

  .empty {
    margin: 0;
    padding: 14px;
    background: var(--surface-muted);
    border-radius: var(--radius-sm);
    border: 1px dashed var(--border);
    color: var(--muted);
  }

  @media (max-width: 720px) {
    .fields {
      gap: 12px;
    }

    .value {
      flex-direction: column;
    }
  }

  .actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .debug-btn {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--muted);
    width: 42px;
    height: 42px;
    border-radius: 12px;
    display: grid;
    place-items: center;
    cursor: pointer;
    transition: all 0.2s;
  }

  .debug-btn:hover {
    color: var(--text);
    background: var(--surface-muted);
  }

  .debug-btn svg {
    width: 20px;
    height: 20px;
  }

  .debug-panel {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 12px;
    margin-bottom: 12px;
  }

  .debug-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
  }

  .debug-header h3 {
    margin: 0;
    font-size: 1rem;
  }

  .path-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .path-list li {
    display: flex;
    gap: 10px;
    padding: 8px;
    border-radius: 8px;
    background: var(--surface-muted);
    font-size: 0.9rem;
    align-items: flex-start;
  }

  .path-list li.valid {
    border-left: 3px solid #10b981;
  }

  .path-list li.invalid {
    border-left: 3px solid #ef4444;
  }

  .path-details {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: hidden;
  }

  .path {
    font-family: monospace;
    word-break: break-all;
  }

  .error {
    font-size: 0.8rem;
    color: var(--muted);
  }

  .icon-success {
    width: 20px;
    height: 20px;
    color: #10b981;
  }

  .icon-error {
    width: 20px;
    height: 20px;
    color: #ef4444;
  }
</style>
