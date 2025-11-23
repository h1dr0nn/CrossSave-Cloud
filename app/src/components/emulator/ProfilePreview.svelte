<script lang="ts">
  import { pushInfo, pushError } from "../../lib/notifications";
  import { openFolder as openFolderAPI } from "../../lib/api";
  import type { EmulatorProfile } from "../lib/api";

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
        await openFolderAPI(path);
        pushInfo(`Opening folder: ${path}`);
        return;
      } catch (error) {
        console.warn(`Failed to open ${path}`, error);
      }
    }

    pushError(
      "Could not open any of the configured folders. Please check if they exist."
    );
  }
</script>

<section class="panel">
  <header>
    <div>
      <p class="eyebrow">Save folder preview</p>
      <h2>{profile?.name ?? "Select an emulator"}</h2>
    </div>
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
  </header>

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

  /* Debug styles removed */
</style>
