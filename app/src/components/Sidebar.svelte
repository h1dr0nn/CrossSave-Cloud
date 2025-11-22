<script lang="ts">
  import { onMount } from "svelte";
  import { createEventDispatcher } from "svelte";
  import EmulatorList from "./EmulatorList.svelte";
  import type { EmulatorProfile } from "../lib/api";

  export let emulators: EmulatorProfile[] = [];
  export let selectedId: string | null = null;
  export let loading = false;
  export let isMobile = false;
  export let isOpen = false;

  const dispatch = createEventDispatcher<{
    select: string;
    close: void;
  }>();

  function selectEmulator(id: string) {
    dispatch("select", id);
  }

  function close() {
    dispatch("close");
  }

  // Lock body scroll when sidebar is open on mobile
  $: if (typeof document !== "undefined") {
    if (isMobile && isOpen) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "";
    }
  }
</script>

{#if isMobile && isOpen}
  <div
    class="overlay"
    on:click={close}
    on:keydown={(e) => e.key === "Escape" && close()}
    role="button"
    tabindex="-1"
    aria-label="Close sidebar"
  ></div>
{/if}

<aside
  class:selected={!isMobile || isOpen}
  class:drawer={isMobile}
  aria-label="Emulator list"
>
  <div class="sidebar-surface">
    <div class="sidebar-header">
      <div class="title">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          aria-hidden="true"
        >
          <rect
            x="6"
            y="4"
            width="12"
            height="16"
            rx="2"
            fill="url(#sidebar-grad)"
          />
          <rect
            x="7.5"
            y="5.5"
            width="9"
            height="5"
            rx="0.5"
            fill="#1e293b"
            opacity="0.3"
          />
          <circle cx="9" cy="14" r="1" fill="#10b981" />
          <circle cx="9" cy="17" r="1" fill="#10b981" />
          <path d="M14 13h1v1h1v1h-1v1h-1v-1h-1v-1h1v-1z" fill="#22c55e" />
          <circle cx="15.5" cy="18" r="0.7" fill="#ef4444" />
          <circle cx="13.5" cy="18" r="0.7" fill="#3b82f6" />
          <defs>
            <linearGradient id="sidebar-grad" x1="6" y1="4" x2="18" y2="20">
              <stop offset="0%" stop-color="#8b5cf6" />
              <stop offset="100%" stop-color="#6366f1" />
            </linearGradient>
          </defs>
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
        on:select={(e) => selectEmulator(e.detail)}
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
    z-index: 6;
    overflow: hidden;
  }

  aside.drawer {
    border-right: none;
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
    border: 1px solid var(--border);
    border-radius: 16px;
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
