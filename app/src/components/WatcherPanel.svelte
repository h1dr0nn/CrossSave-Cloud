<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type { FsEventPayload } from "../lib/api";
  import { startWatcher, stopWatcher, subscribeFsEvents } from "../lib/api";

  let pathsInput = "";
  let logs: string[] = [];
  let statusMessage = "";
  let unsubscribe: (() => void) | null = null;

  const parsePaths = () =>
    pathsInput
      .split("\n")
      .map((value) => value.trim())
      .filter(Boolean);

  const appendLog = (payload: FsEventPayload) => {
    const timestamp = payload.timestamp ? ` @ ${payload.timestamp}` : "";
    const entry = `[${payload.kind}] ${payload.path}${timestamp}`;
    logs = [entry, ...logs].slice(0, 200);
  };

  const startListening = async () => {
    unsubscribe = await subscribeFsEvents((payload) => appendLog(payload));
  };

  onMount(() => {
    startListening();
  });

  onDestroy(() => {
    unsubscribe?.();
  });

  const handleStart = async () => {
    statusMessage = "";
    try {
      await startWatcher(parsePaths());
      statusMessage = "Watcher started";
    } catch (error) {
      statusMessage = `Start failed: ${error}`;
    }
  };

  const handleStop = async () => {
    statusMessage = "";
    try {
      await stopWatcher();
      statusMessage = "Watcher stopped";
    } catch (error) {
      statusMessage = `Stop failed: ${error}`;
    }
  };
</script>

<section class="panel-content">
  <div class="field">
    <label for="paths">Paths (one per line)</label>
    <textarea id="paths" rows="4" bind:value={pathsInput} placeholder="/home/user/saves
/home/user/.config/game"></textarea>
  </div>

  <div class="actions">
    <button on:click={handleStart}>Start Watcher</button>
    <button on:click={handleStop}>Stop Watcher</button>
    {#if statusMessage}
      <span class="status">{statusMessage}</span>
    {/if}
  </div>

  <div class="logs">
    <p class="section-title">File Events</p>
    {#if logs.length === 0}
      <p class="placeholder">No events yet.</p>
    {:else}
      <ul>
        {#each logs as log, index}
          <li>{logs.length - index}. {log}</li>
        {/each}
      </ul>
    {/if}
  </div>
</section>

<style>
  .panel-content {
    display: flex;
    flex-direction: column;
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

  textarea {
    padding: 8px;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    font-family: monospace;
    min-height: 80px;
  }

  .actions {
    display: flex;
    align-items: center;
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

  .status {
    font-size: 13px;
    color: #0f172a;
  }

  .logs {
    border-top: 1px solid #e2e8f0;
    padding-top: 8px;
  }

  .logs ul {
    padding-left: 16px;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 240px;
    overflow-y: auto;
    font-family: monospace;
    font-size: 13px;
  }

  .placeholder {
    color: #94a3b8;
    font-size: 13px;
  }
</style>
