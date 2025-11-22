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

<aside
  class:selected={!isMobile || open}
  class:drawer={isMobile}
  aria-label="Emulator list"
>
  <div class="sidebar-surface">
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
          <small
            >{loading
              ? "Loading profiles"
              : `${emulators.length} available`}</small
          >
        </div>
      </div>
      {#if isMobile}
        <button
          class="close"
          on:click={close}
          aria-label="Close emulator drawer"
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path
              d="M6 6l12 12m0-12L6 18"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
            />
          </svg>
        </button>
      {/if}
    </div>

    <div class="list-shell">
      <EmulatorList
        {emulators}
        {selectedId}
        on:select={(event) => select(event.detail)}
      />
    </div>
  </div>
</aside>

<style>
  aside {
    position: sticky;
    top: 0;
    align-self: start;
    height: 100vh;
    width: clamp(260px, 24vw, 300px);
    min-width: 260px;
    max-width: 320px;
    padding: 12px;
    background: transparent;
    border-right: 1px solid rgba(var(--border-rgb), 0.5);
    border-right: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    z-index: 6;
    overflow: hidden;
  }

  .sidebar-surface {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding-top: max(14px, env(safe-area-inset-top));
    padding-bottom: max(14px, env(safe-area-inset-bottom));
    padding-left: max(14px, env(safe-area-inset-left));
    padding-right: 14px;
    background: var(--surface);
    /* backdrop-filter: blur(20px) saturate(1.1); -- Disabled for performance on low-end devices */
    border: 1px solid rgba(var(--border-rgb), 0.5);
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    box-shadow: var(--shadow-soft);
    overflow: hidden;
  }

  aside.drawer {
    position: fixed;
    inset: 0 auto 0 0;
    height: 100vh;
    max-width: 320px;
    width: min(82vw, 320px);
    min-width: 260px;
    border-right: 1px solid var(--border);
    box-shadow: 0 16px 32px var(--shadow);
    background: var(--surface);
    border-radius: 0 18px 18px 0;
    transform: translateX(-105%);
    transition:
      transform 0.25s ease,
      opacity 0.25s ease;
    z-index: 9999;
  }

  aside.drawer .sidebar-surface {
    border-radius: 0 18px 18px 0;
  }

  aside.drawer.selected {
    transform: translateX(0);
    opacity: 1;
  }

  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(var(--overlay-rgb), 0.6);
    background: color-mix(in srgb, var(--overlay) 85%, transparent);
    z-index: 9998;
    backdrop-filter: blur(1px);
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: clamp(8px, 1vw, 12px);
    padding-bottom: 4px;
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
    background: rgba(var(--surface-muted-rgb), 0.8);
    background: color-mix(in srgb, var(--surface-muted) 90%, transparent);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: clamp(6px, 1vw, 8px);
    cursor: pointer;
    color: var(--text);
    box-shadow: var(--shadow-soft);
  }

  .close svg {
    width: clamp(18px, 4vw, 22px);
    height: clamp(18px, 4vw, 22px);
  }

  .list-shell {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    padding-right: 4px;
    scrollbar-width: none;
  }

  .list-shell::-webkit-scrollbar {
    display: none;
  }

  @media (max-width: 760px) {
    aside {
      width: 100%;
      max-width: none;
      min-width: 0;
      padding: 0;
      border-right: none;
    }
  }
</style>
