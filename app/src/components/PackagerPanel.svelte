<script lang="ts">
  import type { PackagedSave } from "../lib/api";
  import { packageSave } from "../lib/api";

  let gameId = "";
  let emulatorId = "";
  let pathsInput = "";
  let patternsInput = "";
  let result: PackagedSave | null = null;
  let errorMessage = "";

  const parseLines = (value: string) =>
    value
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean);

  const handlePackage = async () => {
    errorMessage = "";
    result = null;

    try {
      result = await packageSave(gameId.trim(), emulatorId.trim(), parseLines(pathsInput), parseLines(patternsInput));
    } catch (error) {
      errorMessage = `Package failed: ${error}`;
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
      <input id="emulator" type="text" bind:value={emulatorId} placeholder="citra" />
    </div>
  </div>

  <div class="field">
    <label for="paths">Paths (one per line)</label>
    <textarea id="paths" rows="4" bind:value={pathsInput} placeholder="/home/user/saves
/home/user/.config/game"></textarea>
  </div>

  <div class="field">
    <label for="patterns">Patterns (glob, one per line)</label>
    <textarea id="patterns" rows="3" bind:value={patternsInput} placeholder="**/*.sav
**/*.bin"></textarea>
  </div>

  <div class="actions">
    <button on:click={handlePackage}>Package Save</button>
    {#if errorMessage}
      <span class="error">{errorMessage}</span>
    {/if}
  </div>

  <div class="output">
    <p class="section-title">Metadata</p>
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

  .error {
    color: #b91c1c;
    font-size: 13px;
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
