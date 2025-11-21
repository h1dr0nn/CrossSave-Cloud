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
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: clamp(12px, 1vw, 16px);
    padding: clamp(14px, 2vw, 20px);
    box-shadow: var(--shadow-strong);
    display: flex;
    flex-direction: column;
    gap: clamp(12px, 1.2vw, 18px);
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
    font-size: clamp(1.2rem, 0.8vw + 1rem, 1.45rem);
    color: var(--text);
  }

  .eyebrow {
    margin: 0;
    color: var(--muted);
    font-size: clamp(0.85rem, 0.3vw + 0.75rem, 0.95rem);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .open {
    display: inline-flex;
    align-items: center;
    gap: clamp(6px, 1vw, 10px);
    padding: clamp(10px, 1.2vw, 14px) clamp(12px, 1.6vw, 16px);
    border-radius: clamp(10px, 1vw, 14px);
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
    width: clamp(18px, 3vw, 22px);
    height: clamp(18px, 3vw, 22px);
  }

  .fields {
    display: flex;
    flex-direction: column;
    gap: clamp(10px, 1vw, 14px);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: clamp(4px, 0.8vw, 8px);
  }

  .label {
    font-weight: 700;
    color: var(--text);
    font-size: clamp(0.95rem, 0.4vw + 0.85rem, 1.05rem);
    margin: 0;
  }

  .value {
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: clamp(10px, 1.5vw, 14px);
    background: var(--surface-muted);
    white-space: pre-wrap;
    color: var(--text);
    min-height: clamp(48px, 6vw, 64px);
    display: flex;
    flex-wrap: wrap;
    gap: clamp(6px, 1vw, 10px);
  }

  code {
    background: var(--card-contrast);
    color: var(--accent-muted);
    padding: clamp(4px, 0.7vw, 6px) clamp(8px, 1.2vw, 10px);
    border-radius: clamp(8px, 1vw, 12px);
    font-size: clamp(0.85rem, 0.3vw + 0.75rem, 0.95rem);
  }

  .preview {
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: clamp(12px, 1.5vw, 16px);
    display: grid;
    grid-template-columns: 1fr;
    gap: clamp(8px, 1vw, 12px);
    background: var(--surface-muted);
  }

  .preview strong {
    font-size: clamp(1rem, 0.4vw + 0.9rem, 1.1rem);
    color: var(--text);
  }

  .preview ul {
    margin: 0;
    padding-left: clamp(14px, 2vw, 18px);
    display: grid;
    gap: clamp(6px, 1vw, 10px);
  }

  .preview li {
    color: var(--muted);
  }

  .empty {
    margin: 0;
    padding: clamp(12px, 2vw, 16px);
    background: var(--surface-muted);
    border-radius: 12px;
    border: 1px dashed var(--border);
    color: var(--muted);
  }

  @media (max-width: 720px) {
    .fields {
      gap: clamp(12px, 2vw, 16px);
    }

    .value {
      flex-direction: column;
    }
  }
</style>
