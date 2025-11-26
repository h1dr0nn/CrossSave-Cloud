<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import { deleteHistoryItem, rollbackVersion } from "../../lib/api";
  import { pushError, pushInfo } from "../../lib/notifications";
  import { formatErrorMessage } from "../../lib/errorMessages";
  import type { HistoryEntry } from "../../lib/api";

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
    minute: "2-digit",
  });

  let workingId: string | null = null;

  function formatTimestamp(timestamp: number) {
    return formatter.format(timestamp * 1000);
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
      pushError(formatErrorMessage(error));
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
      pushError(formatErrorMessage(error));
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
    <p class="placeholder">
      No history yet. Package a save to start tracking versions.
    </p>
  {:else}
    <ul class="history-list">
      {#each history.slice(0, 10) as entry}
        <li class="history-item">
          <div
            class="item-header"
            on:click={() => dispatch("select", entry)}
            on:keypress={() => dispatch("select", entry)}
            role="button"
            tabindex="0"
          >
            <p class="version" title={entry.metadata.version_id}>
              Version {entry.metadata.version_id}
            </p>
            <p class="timestamp">
              {formatTimestamp(entry.metadata.timestamp)}
            </p>
          </div>
          <div class="divider"></div>
          <div class="actions">
            <button
              class="ghost"
              disabled={workingId === entry.metadata.version_id}
              on:click={() => restore(entry.metadata.version_id)}
            >
              Restore
            </button>
            <button
              class="danger"
              disabled={workingId === entry.metadata.version_id}
              on:click={() => remove(entry.metadata.version_id)}
            >
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
    min-height: 320px;
    max-height: 320px;
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
    max-height: 230px;
    overflow-y: auto;
    overflow-x: hidden;
    padding-right: 4px;
  }

  /* Scrollbar styling */
  .history-list::-webkit-scrollbar {
    width: 8px;
  }

  .history-list::-webkit-scrollbar-track {
    background: color-mix(in srgb, var(--border) 30%, transparent);
    border-radius: 4px;
  }

  .history-list::-webkit-scrollbar-thumb {
    background: color-mix(in srgb, var(--border) 80%, transparent);
    border-radius: 4px;
  }

  .history-list::-webkit-scrollbar-thumb:hover {
    background: var(--border);
  }

  .history-item {
    border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
    border-radius: 14px;
    background: color-mix(in srgb, var(--surface-muted) 70%, var(--surface));
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .item-header {
    padding: 10px 12px;
    cursor: pointer;
    transition: background 0.2s;
    border-radius: 14px 14px 0 0;
  }

  .item-header:hover {
    background: color-mix(in srgb, var(--accent-muted) 20%, transparent);
  }

  .version {
    margin: 0 0 4px 0;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .timestamp {
    margin: 0;
    color: var(--muted);
    font-size: 0.85rem;
  }

  .divider {
    height: 1px;
    background: color-mix(in srgb, var(--border) 50%, transparent);
    margin: 0;
  }

  .hash {
    font-family: "SFMono-Regular", ui-monospace, SFMono-Regular, Menlo, Monaco,
      Consolas, "Liberation Mono", "Courier New", monospace;
    color: var(--text);
    padding: 4px 8px;
    border-radius: 10px;
    background: color-mix(in srgb, var(--surface) 60%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
  }

  .actions {
    display: flex;
    gap: 8px;
    padding: 10px 12px;
    justify-content: center;
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
</style>
