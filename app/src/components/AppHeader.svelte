<script lang="ts">
  export let title = "";
  export let eyebrow = "";
  export let showMenu = false;
  export let showBack = false;
  export let onMenu: (() => void) | undefined;
  export let onBack: (() => void) | undefined;
</script>

<header class="app-header" aria-label={title}>
  <div class="leading">
    {#if showMenu}
      <button class="icon-button" on:click={onMenu} aria-label="Open navigation menu">
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M4 7h16M4 12h16M4 17h16" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
        </svg>
      </button>
    {:else if showBack}
      <button class="icon-button" on:click={onBack} aria-label="Go back">
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M14.5 6 8.5 12l6 6" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
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
    padding: 14px 18px;
    border-radius: var(--radius);
    background: color-mix(in srgb, var(--surface) 92%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 90%, transparent);
    box-shadow: var(--shadow-soft);
    backdrop-filter: blur(16px);
    position: sticky;
    top: 0;
    z-index: 12;
    min-width: 0;
    overflow: hidden;
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

  .icon-button {
    width: 40px;
    height: 40px;
    border-radius: 14px;
    border: 1px solid var(--border);
    background: var(--surface);
    display: grid;
    place-items: center;
    color: var(--text);
    box-shadow: var(--shadow-soft);
    cursor: pointer;
    transition: transform 0.15s ease, box-shadow 0.2s ease, border-color 0.2s ease;
  }

  .icon-button.primary {
    background: var(--surface-muted);
    backdrop-filter: blur(10px);
  }

  .icon-button:hover {
    transform: translateY(-1px);
    border-color: var(--accent);
    box-shadow: var(--shadow);
  }

  .icon-button svg {
    width: 22px;
    height: 22px;
  }
</style>
