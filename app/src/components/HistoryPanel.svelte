<script lang="ts">
  import type { HistoryEntry, PackagedSave } from "../lib/api";
  import { deleteHistoryItem, getHistoryItem, listHistory, rollbackVersion } from "../lib/api";

  let gameId = "";
  let versionId = "";
  let history: HistoryEntry[] = [];
  let selected: HistoryEntry | null = null;
  let rollbackResult: PackagedSave | null = null;
  let message = "";

  const handleList = async () => {
    message = "";
    selected = null;
    rollbackResult = null;
    try {
      history = await listHistory(gameId.trim());
    } catch (error) {
      message = `List failed: ${error}`;
    }
  };

  const handleGet = async () => {
    message = "";
    rollbackResult = null;
    try {
      selected = await getHistoryItem(gameId.trim(), versionId.trim());
    } catch (error) {
      message = `Get failed: ${error}`;
    }
  };

  const handleRollback = async () => {
    message = "";
    try {
      rollbackResult = await rollbackVersion(gameId.trim(), versionId.trim());
      message = "Rollback completed";
    } catch (error) {
      message = `Rollback failed: ${error}`;
    }
  };

  const handleDelete = async () => {
    message = "";
    try {
      await deleteHistoryItem(gameId.trim(), versionId.trim());
      message = "History entry deleted";
    } catch (error) {
      message = `Delete failed: ${error}`;
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
      <input id="version" type="text" bind:value={versionId} placeholder="timestamp or version" />
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
    {#if history.length}
      <pre>{JSON.stringify(history, null, 2)}</pre>
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
