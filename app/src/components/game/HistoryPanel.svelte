<script lang="ts">
  import { onDestroy } from "svelte";
  import type { HistoryEntry, PackagedSave } from "../../lib/api";
  import {
    deleteHistoryItem,
    getHistoryItem,
    listHistory,
    rollbackVersion,
  } from "../../lib/api";
  import { historyState, setHistory } from "../../lib/historyStore";
  import { pushError, pushInfo } from "../../lib/notifications";

  let gameId = "";
  let versionId = "";
  let selected: HistoryEntry | null = null;
  let rollbackResult: PackagedSave | null = null;
  let message = "";
  let historyData = { gameId: "", entries: [] as HistoryEntry[] };

  const unsubscribe = historyState.subscribe((value) => {
    historyData = value;
    if (!gameId && value.gameId) {
      gameId = value.gameId;
    }
  });

  onDestroy(() => unsubscribe());

  const handleList = async () => {
    message = "";
    selected = null;
    rollbackResult = null;
    try {
      const entries = await listHistory(gameId.trim());
      setHistory(gameId.trim(), entries);
      message = "History loaded";
      pushInfo(`Loaded ${entries.length} history entries`);
    } catch (error) {
      pushError(`History list failed: ${error}`);
    }
  };

  const handleGet = async () => {
    message = "";
    rollbackResult = null;
    try {
      selected = await getHistoryItem(gameId.trim(), versionId.trim());
      pushInfo(`Fetched history entry ${versionId}`);
    } catch (error) {
      pushError(`Get failed: ${error}`);
    }
  };

  const handleRollback = async () => {
    message = "";
    try {
      rollbackResult = await rollbackVersion(gameId.trim(), versionId.trim());
      message = "Rollback completed";
      pushInfo(`Rollback completed for ${versionId}`);
    } catch (error) {
      pushError(`Rollback failed: ${error}`);
    }
  };

  const handleDelete = async () => {
    message = "";
    try {
      await deleteHistoryItem(gameId.trim(), versionId.trim());
      message = "History entry deleted";
      pushInfo(`Deleted ${versionId} for ${gameId}`);
    } catch (error) {
      pushError(`Delete failed: ${error}`);
    }
  };
</script>

<section class="panel-content">
  <div class="grid">
    <div class="field">
      <label for="game">game_id</label>
      <input id="game" type="text" bind:value={gameId} placeholder="my_game" />
    </div>
    <div class="field">
      <label for="version">version_id</label>
      <input
        id="version"
        type="text"
        bind:value={versionId}
        placeholder="timestamp or version"
      />
    </div>
  </div>

  <div class="actions">
    <button on:click={handleList}>List History</button>
    <button on:click={handleGet}>Get Version</button>
    <button on:click={handleRollback}>Rollback</button>
    <button on:click={handleDelete}>Delete</button>
    {#if message}
      <span class="message">{message}</span>
    {/if}
  </div>

  <div class="output">
    <p class="section-title">History List</p>
    {#if historyData.entries.length}
      <pre>{JSON.stringify(historyData.entries, null, 2)}</pre>
    {:else}
      <p class="placeholder">History list is empty.</p>
    {/if}
  </div>

  <div class="output">
    <p class="section-title">Selected Entry</p>
    {#if selected}
      <pre>{JSON.stringify(selected, null, 2)}</pre>
    {:else}
      <p class="placeholder">No entry fetched.</p>
    {/if}
  </div>

  <div class="output">
    <p class="section-title">Rollback Result</p>
    {#if rollbackResult}
      <pre>{JSON.stringify(rollbackResult, null, 2)}</pre>
    {:else}
      <p class="placeholder">No rollback executed.</p>
    {/if}
  </div>
</section>

<style>
  .panel-content {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 12px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  label {
    font-weight: 600;
    font-size: 14px;
  }

  .section-title {
    margin: 0 0 4px 0;
    font-weight: 600;
    font-size: 14px;
  }

  input {
    padding: 8px;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    font-family: monospace;
  }

  .actions {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
  }

  button {
    padding: 8px 12px;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    background: #f8fafc;
    cursor: pointer;
  }

  button:hover {
    background: #e2e8f0;
  }

  .message {
    font-size: 13px;
    color: #0f172a;
  }

  .output pre {
    background: #0f172a;
    color: #e2e8f0;
    padding: 12px;
    border-radius: 6px;
    overflow-x: auto;
  }

  .placeholder {
    color: #94a3b8;
    font-size: 13px;
  }
</style>
