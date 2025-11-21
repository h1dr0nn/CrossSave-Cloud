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
          fill="#0f172a"
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
          <path d="M6 6l12 12m0-12L6 18" stroke="#0f172a" stroke-width="2" stroke-linecap="round" />
        </svg>
      </button>
    {/if}
  </div>

  <EmulatorList emulators={emulators} selectedId={selectedId} on:select={(event) => select(event.detail)} />
</aside>

<style>
  aside {
    position: relative;
    width: clamp(180px, 22vw, 260px);
    min-width: clamp(180px, 20vw, 240px);
    max-width: 260px;
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(6px);
    border-right: 1px solid #e2e8f0;
    box-shadow: 6px 0 24px rgba(15, 23, 42, 0.08);
    padding: clamp(12px, 1.8vw, 18px);
    display: flex;
    flex-direction: column;
    gap: clamp(12px, 1vw, 18px);
    transition: transform 0.25s ease, opacity 0.25s ease;
  }

  aside.drawer {
    position: fixed;
    inset: 0 auto 0 0;
    transform: translateX(-100%);
    max-width: clamp(200px, 60vw, 280px);
    z-index: 40;
    height: 100vh;
  }

  aside.drawer.selected {
    transform: translateX(0);
    opacity: 1;
  }

  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(15, 23, 42, 0.35);
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
  }

  .title p {
    margin: 0;
    font-size: clamp(1rem, 0.5vw + 0.9rem, 1.1rem);
    font-weight: 700;
    color: #0f172a;
  }

  .title small {
    color: #475569;
    font-size: clamp(0.8rem, 0.4vw + 0.65rem, 0.95rem);
  }

  .close {
    background: #e2e8f0;
    border: none;
    border-radius: 10px;
    padding: clamp(6px, 1vw, 8px);
    cursor: pointer;
  }

  .close svg {
    width: clamp(18px, 4vw, 22px);
    height: clamp(18px, 4vw, 22px);
  }
</style>
