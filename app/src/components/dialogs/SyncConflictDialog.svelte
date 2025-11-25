<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let isOpen = false;
  export let gameId = "";
  export let localTimestamp = 0;
  export let cloudTimestamp = 0;

  const dispatch = createEventDispatcher();

  $: if (isOpen) {
    document.body.style.overflow = "hidden";
  } else {
    document.body.style.overflow = "";
  }

  function formatDate(timestamp: number) {
    return new Date(timestamp * 1000).toLocaleString();
  }

  function handleUpload() {
    dispatch("resolve", { action: "upload" });
    isOpen = false;
  }

  function handleDownload() {
    dispatch("resolve", { action: "download" });
    isOpen = false;
  }

  function handleSkip() {
    dispatch("resolve", { action: "skip" });
    isOpen = false;
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal-backdrop" on:click={handleSkip}>
    <div
      class="modal-content"
      on:click|stopPropagation
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <header class="modal-header">
        <h3>Save Conflict Detected</h3>
      </header>

      <div class="modal-body">
        <p class="game-id">Game: <code>{gameId}</code></p>
        <p class="description">
          Both local and cloud have different saves. Choose which one to keep:
        </p>

        <div class="options">
          <button class="option-card" on:click={handleUpload}>
            <div class="option-icon upload">↑</div>
            <div class="option-content">
              <h4>Upload Local Save</h4>
              <p class="timestamp">{formatDate(localTimestamp)}</p>
              <p class="hint">Overwrite cloud with your local save</p>
            </div>
          </button>

          <button class="option-card" on:click={handleDownload}>
            <div class="option-icon download">↓</div>
            <div class="option-content">
              <h4>Download Cloud Save</h4>
              <p class="timestamp">{formatDate(cloudTimestamp)}</p>
              <p class="hint">Overwrite local with cloud save</p>
            </div>
          </button>
        </div>
      </div>

      <footer class="modal-footer">
        <button class="btn-text" on:click={handleSkip}>Decide Later</button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    animation: fadeIn 0.15s ease-out;
    padding: 16px;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .modal-content {
    background: rgb(var(--surface-rgb));
    border: 1px solid var(--border);
    border-radius: var(--radius);
    width: 100%;
    max-width: 500px;
    display: flex;
    flex-direction: column;
    box-shadow: var(--shadow-lg);
    animation: slideUp 0.2s ease-out;
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .modal-header {
    padding: 20px 24px;
    border-bottom: 1px solid var(--border);
    background: rgb(var(--surface-rgb));
    border-radius: var(--radius) var(--radius) 0 0;
  }

  .modal-header h3 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text);
  }

  .modal-body {
    padding: 24px;
    background: rgb(var(--surface-rgb));
  }

  .game-id {
    margin: 0 0 8px 0;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .game-id code {
    background: var(--surface-muted);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: monospace;
    color: var(--text);
  }

  .description {
    margin: 0 0 16px 0;
    color: var(--text-secondary);
    line-height: 1.6;
  }

  .options {
    display: grid;
    gap: 12px;
  }

  .option-card {
    display: flex;
    gap: 16px;
    padding: 16px;
    border: 2px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    cursor: pointer;
    transition: all 0.2s;
    text-align: left;
    width: 100%;
  }

  .option-card:hover {
    border-color: var(--accent);
    background: var(--surface-muted);
    transform: translateY(-2px);
    box-shadow: var(--shadow);
  }

  .option-icon {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    font-size: 24px;
    flex-shrink: 0;
  }

  .option-icon.upload {
    background: linear-gradient(135deg, #10b981, #059669);
    color: white;
  }

  .option-icon.download {
    background: linear-gradient(135deg, #3b82f6, #2563eb);
    color: white;
  }

  .option-content {
    flex: 1;
  }

  .option-content h4 {
    margin: 0 0 4px 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text);
  }

  .timestamp {
    margin: 0 0 4px 0;
    font-size: 0.85rem;
    color: var(--text-secondary);
    font-family: monospace;
  }

  .hint {
    margin: 0;
    font-size: 0.85rem;
    color: var(--muted);
  }

  .modal-footer {
    padding: 16px 24px;
    border-top: 1px solid var(--border);
    display: flex;
    justify-content: flex-end;
    background: rgb(var(--surface-muted-rgb));
    border-radius: 0 0 var(--radius) var(--radius);
  }

  .btn-text {
    background: none;
    border: none;
    padding: 8px 16px;
    cursor: pointer;
    color: var(--text-secondary);
    font-weight: 500;
    transition: color 0.2s;
    border-radius: var(--radius-sm);
  }

  .btn-text:hover {
    color: var(--text);
    background: var(--bg-hover);
  }
</style>
