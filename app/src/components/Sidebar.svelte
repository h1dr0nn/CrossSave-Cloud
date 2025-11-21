<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import EmulatorList from "./EmulatorList.svelte";
  import type { EmulatorProfile } from "../lib/api";

  export let emulators: EmulatorProfile[] = [];
  export let selectedId: string | null = null;
  export let isMobile = false;
  export let open = false;
  export let loading = false;

  const dispatch = createEventDispatcher<{ select: string; close: void }>();

  function select(id: string) {
    dispatch("select", id);
  }

  function close() {
    if (isMobile) {
      dispatch("close");
    }
  }
</script>

{#if isMobile && open}
  <div class="overlay" on:click={close} aria-hidden="true"></div>
{/if}

<aside class:selected={!isMobile || open} class:drawer={isMobile} aria-label="Emulator list">
  <div class="sidebar-header">
    <div class="title">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M6.5 3.5h11a2 2 0 0 1 2 2v13a2 2 0 0 1-2 2h-11a2 2 0 0 1-2-2v-13a2 2 0 0 1 2-2Zm0 2v13h11v-13Z"
          fill="currentColor"
        />
        <circle cx="9" cy="9" r="1" fill="#10b981" />
        <circle cx="15" cy="9" r="1" fill="#10b981" />
      </svg>
      <div>
        <p>Emulators</p>
        <small>{loading ? "Loading profiles" : `${emulators.length} available`}</small>
      </div>
    </div>
    {#if isMobile}
      <button class="close" on:click={close} aria-label="Close emulator drawer">
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M6 6l12 12m0-12L6 18" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
        </svg>
      </button>
    {/if}
  </div>

  <EmulatorList emulators={emulators} selectedId={selectedId} on:select={(event) => select(event.detail)} />
</aside>

<style>
  aside {
    position: sticky;
    top: 0;
    align-self: start;
    height: 100vh;
    width: clamp(220px, 22vw, 280px);
    min-width: clamp(220px, 22vw, 280px);
    max-width: 320px;
    background: color-mix(in srgb, var(--surface) 95%, transparent);
    backdrop-filter: blur(12px);
    border-right: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    box-shadow: var(--shadow-soft);
    border-radius: 0 18px 18px 0;
    overflow: hidden;
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    transition: transform 0.25s ease, opacity 0.25s ease;
    z-index: 2;
  }

  aside.drawer {
    position: fixed;
    inset: 0 auto 0 0;
    height: 100vh;
    transform: translateX(-105%);
    max-width: 320px;
    width: min(82vw, 320px);
    border-right: 1px solid var(--border);
    border-bottom: none;
    box-shadow: 0 16px 32px var(--shadow);
    z-index: 40;
    background: var(--surface);
    border-radius: 0;
  }

  aside.drawer.selected {
    transform: translateX(0);
    opacity: 1;
  }

  .overlay {
    position: fixed;
    inset: 0;
    background: var(--overlay);
    z-index: 30;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: clamp(8px, 1vw, 12px);
  }

  .title {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    gap: clamp(8px, 1vw, 12px);
  }

  .title svg {
    width: clamp(32px, 5vw, 40px);
    height: clamp(32px, 5vw, 40px);
    color: var(--text);
  }

  .title p {
    margin: 0;
    font-size: clamp(1rem, 0.5vw + 0.9rem, 1.1rem);
    font-weight: 700;
    color: var(--text);
  }

  .title small {
    color: var(--muted);
    font-size: clamp(0.8rem, 0.4vw + 0.65rem, 0.95rem);
  }

  .close {
    background: var(--surface-muted);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: clamp(6px, 1vw, 8px);
    cursor: pointer;
    color: var(--text);
  }

  .close svg {
    width: clamp(18px, 4vw, 22px);
    height: clamp(18px, 4vw, 22px);
  }
</style>
