<script lang="ts">
  import type { PackageResponse } from "../../lib/api";
  import { packageSave } from "../../lib/api";
  import { addHistoryEntry } from "../../lib/historyStore";
  import { pushError, pushInfo } from "../../lib/notifications";

  let gameId = "";
  let emulatorId = "";
  let pathsInput = "";
  let patternsInput = "";
  let result: PackageResponse | null = null;
  let packaging = false;

  const parseLines = (value: string) =>
    value
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean);

  const handlePackage = async () => {
    result = null;
    packaging = true;

    try {
      const response = await packageSave(
        gameId.trim(),
        emulatorId.trim(),
        parseLines(pathsInput),
        parseLines(patternsInput)
      );
      result = response;
      addHistoryEntry(response.history);
      pushInfo(
        `Packaged ${response.packaged.metadata.version_id} and saved to history`
      );
    } catch (error) {
      pushError(`Package failed: ${error}`);
    } finally {
      packaging = false;
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
      <label for="emulator">emulator_id</label>
      <input
        id="emulator"
        type="text"
        bind:value={emulatorId}
        placeholder="citra"
      />
    </div>
  </div>

  <div class="field">
    <label for="paths">Paths (one per line)</label>
    <textarea
      id="paths"
      rows="4"
      bind:value={pathsInput}
      placeholder="/home/user/saves
/home/user/.config/game"
    ></textarea>
  </div>

  <div class="field">
    <label for="patterns">Patterns (glob, one per line)</label>
    <textarea
      id="patterns"
      rows="3"
      bind:value={patternsInput}
      placeholder="**/*.sav
**/*.bin"
    ></textarea>
  </div>

  <div class="actions">
    <button on:click={handlePackage} disabled={packaging}>Package Save</button>
    {#if packaging}
      <span class="status">Packaging...</span>
    {/if}
  </div>

  <div class="output">
    <p class="section-title">Package + History Result</p>
    {#if result}
      <pre>{JSON.stringify(result, null, 2)}</pre>
    {:else}
      <p class="placeholder">No package result yet.</p>
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

  input,
  textarea {
    padding: 8px;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    font-family: monospace;
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
