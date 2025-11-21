<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import {
    deleteHistoryItem,
    rollbackVersion,
    type HistoryEntry
  } from "../lib/api";
  import { pushError, pushInfo } from "../lib/notifications";

  export let gameId: string;
  export let history: HistoryEntry[] = [];
  export let loading = false;

  const dispatch = createEventDispatcher<{
    select: HistoryEntry;
    reload: void;
  }>();

  const formatter = new Intl.DateTimeFormat(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit"
  });

  let workingId: string | null = null;

  function formatTimestamp(timestamp: number) {
    return formatter.format(timestamp);
  }

  function shortHash(hash: string) {
    return hash?.slice(0, 8) ?? "";
  }

  function estimateSize(entry: HistoryEntry) {
    const bytes = entry.metadata.file_list.length * 128 * 1024;
    const units = ["B", "KB", "MB", "GB"];
    let value = bytes;
    let unitIndex = 0;

    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex += 1;
    }

    return `${value.toFixed(1)} ${units[unitIndex]}`;
  }

  async function restore(versionId: string) {
    workingId = versionId;
    try {
      await rollbackVersion(gameId, versionId);
      pushInfo(`Restored version ${versionId}`);
      dispatch("reload");
    } catch (error) {
      pushError(`Restore failed: ${error}`);
    } finally {
      workingId = null;
    }
  }

  async function remove(versionId: string) {
    workingId = versionId;
    try {
      await deleteHistoryItem(gameId, versionId);
      pushInfo(`Deleted version ${versionId}`);
      dispatch("reload");
    } catch (error) {
      pushError(`Delete failed: ${error}`);
    } finally {
      workingId = null;
    }
  }
</script>

<section class="panel">
  <header>
    <div>
      <p class="eyebrow">Recent History</p>
      <h2>{Math.min(history.length, 10)} of {history.length} versions</h2>
    </div>
  </header>

  {#if loading}
    <p class="placeholder">Loading history...</p>
  {:else if history.length === 0}
    <p class="placeholder">No history yet. Package a save to start tracking versions.</p>
  {:else}
    <ul class="history-list">
      {#each history.slice(0, 10) as entry}
        <li>
          <button class="history-row" on:click={() => dispatch("select", entry)}>
            <div class="meta">
              <p class="version">Version {entry.metadata.version_id}</p>
              <p class="timestamp">{formatTimestamp(entry.metadata.timestamp)}</p>
            </div>
            <div class="stats">
              <span>{entry.metadata.file_list.length} files</span>
              <span>{estimateSize(entry)}</span>
              <span class="hash">{shortHash(entry.metadata.hash)}</span>
            </div>
          </button>
          <div class="actions">
            <button class="ghost" disabled={workingId === entry.metadata.version_id} on:click={() => restore(entry.metadata.version_id)}>
              Restore
            </button>
            <button class="danger" disabled={workingId === entry.metadata.version_id} on:click={() => remove(entry.metadata.version_id)}>
              Delete
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .panel {
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    background: color-mix(in srgb, var(--surface) 90%, transparent);
    border-radius: var(--radius);
    padding: 14px 16px;
    box-shadow: var(--shadow-soft);
    min-height: 0;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
  }

  h2 {
    margin: 4px 0 0;
    font-size: 1.1rem;
  }

  .eyebrow {
    margin: 0;
    color: var(--muted);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    font-size: 0.8rem;
  }

  .placeholder {
    margin: 10px 0;
    color: var(--muted);
  }

  .history-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  li {
    border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
    border-radius: 14px;
    background: color-mix(in srgb, var(--surface-muted) 70%, var(--surface));
    overflow: hidden;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
  }

  .history-row {
    width: 100%;
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 10px;
    padding: 12px 14px;
    border: none;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }

  .history-row:hover {
    background: color-mix(in srgb, var(--accent-muted) 25%, transparent);
  }

  .meta {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .version {
    margin: 0;
    font-weight: 700;
  }

  .timestamp {
    margin: 0;
    color: var(--muted);
  }

  .stats {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
    justify-content: flex-end;
    color: var(--muted);
    font-size: 0.95rem;
  }

  .hash {
    font-family: "SFMono-Regular", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas,
      "Liberation Mono", "Courier New", monospace;
    color: var(--text);
    padding: 4px 8px;
    border-radius: 10px;
    background: color-mix(in srgb, var(--surface) 60%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
  }

  .actions {
    display: flex;
    gap: 8px;
    padding: 0 12px 12px;
  }

  .ghost,
  .danger {
    border-radius: 10px;
    padding: 8px 12px;
    border: 1px solid var(--border);
    background: var(--surface);
    cursor: pointer;
  }

  .ghost:disabled,
  .danger:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .danger {
    border-color: color-mix(in srgb, #ef4444 40%, var(--border));
    color: #b91c1c;
  }

  @media (max-width: 720px) {
    .history-row {
      grid-template-columns: 1fr;
    }

    .stats {
      justify-content: flex-start;
    }
  }
</style>
