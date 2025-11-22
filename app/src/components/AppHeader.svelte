<script lang="ts">
  export let title = "";
  export let eyebrow = "";
  export let showMenu = false;
  export let showBack = false;
  export let onMenu: (() => void) | undefined;
  export let onBack: (() => void) | undefined;
  export let sticky = true;
</script>

<header class="app-header" class:sticky aria-label={title}>
  <div class="leading">
    {#if showMenu}
      <button
        class="icon-button"
        on:click={onMenu}
        aria-label="Open navigation menu"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path
            d="M4 7h16M4 12h16M4 17h16"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
          />
        </svg>
      </button>
    {:else if showBack}
      <button class="icon-button" on:click={onBack} aria-label="Go back">
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path
            d="M14.5 6 8.5 12l6 6"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
          />
        </svg>
      </button>
    {/if}

    <div class="titles">
      {#if eyebrow}
        <p class="eyebrow">{eyebrow}</p>
      {/if}
      <h1>{title}</h1>
    </div>
  </div>

  <div class="actions">
    <slot name="actions"></slot>
  </div>
</header>

<style>
  .app-header {
    display: grid;
    grid-template-columns: 1fr auto;
    align-items: center;
    gap: 12px;
    min-height: 80px;
    padding: max(10px, env(safe-area-inset-top))
      max(20px, env(safe-area-inset-right)) clamp(10px, 2vw, 14px)
      max(20px, env(safe-area-inset-left));
    border-radius: var(--radius);
    background: rgba(var(--surface-rgb), 0.8);
    background: color-mix(in srgb, var(--surface) 92%, transparent);
    border: 1px solid rgba(var(--border-rgb), 0.5);
    border: 1px solid color-mix(in srgb, var(--border) 90%, transparent);
    box-shadow: var(--shadow-soft);
    backdrop-filter: blur(16px);
    min-width: 0;
    overflow: hidden;
  }

  .app-header.sticky {
    position: sticky;
    top: 0;
    z-index: 12;
  }

  .leading {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .titles {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  h1 {
    margin: 0;
    font-size: clamp(1.2rem, 0.5vw + 1.05rem, 1.6rem);
    font-weight: 700;
    color: var(--text);
    letter-spacing: -0.02em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .eyebrow {
    margin: 0;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--muted);
    font-weight: 600;
  }

  .actions {
    display: flex;
    gap: 10px;
    align-items: center;
  }
</style>
