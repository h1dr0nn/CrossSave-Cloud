<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from "svelte";

  export let scanning = false;
  export let progress = "";

  const dispatch = createEventDispatcher<{ scan: void; cancel: void }>();

  // Lock body scroll when popup is mounted
  onMount(() => {
    document.body.style.overflow = "hidden";
  });

  onDestroy(() => {
    document.body.style.overflow = "";
  });
</script>

<div class="backdrop">
  <div class="popup" role="dialog" aria-modal="true">
    {#if scanning}
      <div class="content scanning">
        <div class="spinner">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <circle
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              stroke-width="3"
              stroke-opacity="0.25"
            />
            <path
              d="M12 2C6.47715 2 2 6.47715 2 12"
              stroke="currentColor"
              stroke-width="3"
              stroke-linecap="round"
            />
          </svg>
        </div>
        <h3>Scanning Library</h3>
        <p>{progress || "Please wait..."}</p>
      </div>
    {:else}
      <div class="content">
        <h3>Welcome to CrossSave Cloud!</h3>
        <p>Would you like to scan all systems for save files?</p>
        <div class="actions">
          <button class="btn secondary" on:click={() => dispatch("cancel")}>
            Skip
          </button>
          <button class="btn primary" on:click={() => dispatch("scan")}>
            Scan All Systems
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
    display: grid;
    place-items: center;
    z-index: 100;
    padding: 20px;
  }

  .popup {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 24px;
    padding: 32px;
    width: min(400px, 100%);
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
  }

  .content {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 16px;
  }

  h3 {
    margin: 0;
    font-size: 1.5rem;
    color: var(--text);
  }

  p {
    margin: 0;
    color: var(--muted);
    line-height: 1.5;
  }

  .actions {
    display: flex;
    gap: 12px;
    width: 100%;
    margin-top: 8px;
  }

  .btn {
    flex: 1;
    padding: 14px;
    border-radius: 14px;
    font-weight: 600;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
    border: none;
  }

  .btn.primary {
    background: var(--accent);
    color: white;
  }

  .btn.primary:hover {
    filter: brightness(1.1);
  }

  .btn.secondary {
    background: var(--surface-muted);
    color: var(--text);
  }

  .btn.secondary:hover {
    background: color-mix(in srgb, var(--surface-muted) 80%, var(--text));
  }

  .scanning {
    padding: 20px 0;
  }

  .spinner {
    width: 48px;
    height: 48px;
    color: var(--accent);
    animation: spin 1s linear infinite;
    margin-bottom: 8px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
