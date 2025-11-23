<script lang="ts">
  import { flip } from "svelte/animate";
  import { fade, fly } from "svelte/transition";
  import { notifications, dismissNotification } from "../../lib/notifications";

  $: items = $notifications;
</script>

<div class="notifications-container">
  {#each items as item (item.id)}
    <div
      class="toast"
      class:error={item.level === "error"}
      class:info={item.level === "info"}
      animate:flip={{ duration: 300 }}
      in:fly={{ y: 20, duration: 300 }}
      out:fade={{ duration: 200 }}
      role="alert"
    >
      <div class="content">
        {#if item.level === "error"}
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="icon"
          >
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
        {:else}
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="icon"
          >
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="16" x2="12" y2="12" />
            <line x1="12" y1="8" x2="12.01" y2="8" />
          </svg>
        {/if}
        <p>{item.message}</p>
      </div>
      <button
        on:click={() => dismissNotification(item.id)}
        aria-label="Dismiss"
      >
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>
  {/each}
</div>

<style>
  .notifications-container {
    position: fixed;
    bottom: 24px;
    right: 24px;
    z-index: 10000;
    display: flex;
    flex-direction: column-reverse;
    gap: 10px;
    pointer-events: none;
    max-width: 400px;
    width: calc(100% - 48px);
  }

  .toast {
    pointer-events: auto;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 12px 16px;
    box-shadow: var(--shadow-soft);
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    color: var(--text);
    font-size: 0.95rem;
  }

  .toast.error {
    background: color-mix(in srgb, #ef4444 10%, var(--surface));
    border-color: color-mix(in srgb, #ef4444 30%, var(--border));
  }

  .toast.error .icon {
    color: #ef4444;
  }

  .toast.info {
    background: color-mix(in srgb, #3b82f6 10%, var(--surface));
    border-color: color-mix(in srgb, #3b82f6 30%, var(--border));
  }

  .toast.info .icon {
    color: #3b82f6;
  }

  .content {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .content p {
    margin: 0;
    line-height: 1.4;
  }

  .icon {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  button {
    background: transparent;
    border: none;
    padding: 4px;
    cursor: pointer;
    color: var(--muted);
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  button:hover {
    background: rgba(0, 0, 0, 0.05);
    color: var(--text);
  }

  button svg {
    width: 16px;
    height: 16px;
  }

  @media (max-width: 600px) {
    .notifications-container {
      bottom: 20px;
      right: 20px;
      left: 20px;
      width: auto;
    }
  }
</style>
