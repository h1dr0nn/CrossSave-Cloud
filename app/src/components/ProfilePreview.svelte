<script lang="ts">
  import { openPath } from "@tauri-apps/plugin-opener";
  import type { EmulatorProfile } from "../lib/api";

  export let profile: EmulatorProfile | null = null;
  export let loading = false;

  $: matchedCount = profile
    ? Math.max(profile.file_patterns.length * Math.max(profile.default_save_paths.length, 1), 0)
    : 0;

  async function openFolder() {
    if (!profile?.default_save_paths.length) return;
    try {
      await openPath(profile.default_save_paths[0]);
    } catch (error) {
      console.error("Unable to open folder", error);
    }
  }
</script>

<section class="panel">
  <header>
    <div>
      <p class="eyebrow">Save folder preview</p>
      <h2>{profile?.name ?? "Select an emulator"}</h2>
    </div>
    <button class="open" on:click={openFolder} disabled={!profile?.default_save_paths.length}>
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
        <p class="label">Glob patterns</p>
        <div class="value">
          {#each profile.file_patterns as pattern}
            <code>{pattern}</code>
          {/each}
        </div>
      </div>
    </div>

    <div class="preview">
      <div>
        <p class="eyebrow">Preview</p>
        <strong>{matchedCount} matched files</strong>
      </div>
      <ul>
        {#each profile.file_patterns as pattern}
          <li>{pattern}</li>
        {/each}
      </ul>
    </div>
  {/if}
</section>

<style>
  .panel {
    background: linear-gradient(180deg, color-mix(in srgb, var(--surface) 94%, transparent), var(--surface));
    border: 1px solid color-mix(in srgb, var(--border) 90%, transparent);
    border-radius: var(--radius);
    padding: 16px;
    box-shadow: var(--shadow-soft);
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-height: 0;
    color: var(--text);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
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
    transition: transform 0.15s ease, box-shadow 0.2s ease, opacity 0.2s ease;
  }

  .open:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    box-shadow: none;
  }

  .open:not(:disabled):hover {
    transform: translateY(-1px);
    box-shadow: 0 12px 24px color-mix(in srgb, var(--accent-strong) 30%, transparent);
  }

  .open svg {
    width: 20px;
    height: 20px;
  }

  .fields {
    display: flex;
    flex-direction: column;
    gap: 10px;
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
    background: var(--surface-muted);
    white-space: pre-wrap;
    color: var(--text);
    min-height: 56px;
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  code {
    background: var(--card-contrast);
    color: var(--accent-muted);
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    font-size: 0.88rem;
  }

  .preview {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 12px;
    display: grid;
    grid-template-columns: 1fr;
    gap: 8px;
    background: var(--surface-muted);
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
</style>
