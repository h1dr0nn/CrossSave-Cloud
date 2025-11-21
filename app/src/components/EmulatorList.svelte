<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { EmulatorProfile } from "../lib/api";

  export let emulators: EmulatorProfile[] = [];
  export let selectedId: string | null = null;

  const dispatch = createEventDispatcher<{ select: string }>();

  function selectEmulator(id: string) {
    dispatch("select", id);
  }
</script>

<div class="list" role="listbox" aria-label="Emulator profiles">
  {#if emulators.length === 0}
    <p class="empty">No emulator profiles found.</p>
  {:else}
    {#each emulators as emulator}
      <button
        class:selected={emulator.emulator_id === selectedId}
        on:click={() => selectEmulator(emulator.emulator_id)}
        role="option"
        aria-selected={emulator.emulator_id === selectedId}
      >
        <span class="blur-plate" aria-hidden="true"></span>
        <div class="button-surface">
          <div class="icon">
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path
                d="M5 7.5A2.5 2.5 0 0 1 7.5 5h9A2.5 2.5 0 0 1 19 7.5v9A2.5 2.5 0 0 1 16.5 19h-9A2.5 2.5 0 0 1 5 16.5Zm2.5-.5a.5.5 0 0 0-.5.5v9a.5.5 0 0 0 .5.5h9a.5.5 0 0 0 .5-.5v-9a.5.5 0 0 0-.5-.5Z"
                fill="currentColor"
              />
              <path d="M9.75 11.25h4.5v1.5h-4.5Z" fill="#10b981" />
              <path d="M9.75 14.25h2.5v1.5h-2.5Z" fill="#10b981" />
            </svg>
          </div>
          <div class="text">
            <span>{emulator.name}</span>
            <small>{emulator.default_save_paths[0] ?? "No save path"}</small>
          </div>
        </div>
      </button>
    {/each}
  {/if}
</div>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: clamp(6px, 1vw, 10px);
    overflow: visible;
    padding: clamp(6px, 1.4vw, 12px) clamp(2px, 0.6vw, 6px) 6px 0;
  }

  button {
    position: relative;
    border: none;
    background: transparent;
    border-radius: 14px;
    padding: 0;
    display: block;
    text-align: left;
    cursor: pointer;
    transition: transform 0.15s ease;
    color: var(--text);
  }

  .blur-plate {
    position: absolute;
    inset: -12px -14px;
    background: color-mix(in srgb, var(--surface) 86%, transparent);
    border-radius: 16px;
    filter: blur(18px);
    backdrop-filter: blur(20px);
    opacity: 0.9;
    pointer-events: none;
    z-index: 0;
    transition: opacity 0.2s ease, filter 0.2s ease;
  }

  .button-surface {
    position: relative;
    display: grid;
    grid-template-columns: auto 1fr;
    gap: clamp(10px, 1.5vw, 14px);
    align-items: center;
    padding: clamp(10px, 2vw, 14px);
    border-radius: 14px;
    border: 1px solid var(--border);
    background: linear-gradient(180deg, color-mix(in srgb, var(--surface) 92%, transparent), var(--surface));
    box-shadow: var(--shadow-soft);
    overflow: visible;
    z-index: 1;
  }

  button.selected {
    transform: none;
  }

  button.selected .button-surface {
    border-color: var(--accent);
    background: linear-gradient(120deg, color-mix(in srgb, var(--accent-muted) 80%, var(--surface) 20%), var(--surface));
    box-shadow: 0 10px 20px color-mix(in srgb, var(--accent-strong) 28%, transparent);
    transform: translateY(-1px);
  }

  button:hover .button-surface {
    border-color: color-mix(in srgb, var(--accent) 60%, var(--border));
  }

  button:hover .blur-plate {
    opacity: 1;
    filter: blur(16px);
  }

  .icon {
    width: clamp(36px, 5vw, 44px);
    height: clamp(36px, 5vw, 44px);
    border-radius: 12px;
    display: grid;
    place-items: center;
    color: var(--text);
    background: var(--surface-muted);
  }

  .icon svg {
    width: 100%;
    height: 100%;
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: clamp(4px, 0.8vw, 8px);
    min-width: 0;
  }

  .text span {
    font-weight: 700;
    font-size: clamp(1rem, 0.5vw + 0.9rem, 1.05rem);
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .text small {
    color: var(--muted);
    font-size: clamp(0.8rem, 0.3vw + 0.7rem, 0.95rem);
    display: block;
  }

  .empty {
    margin: 0;
    padding: clamp(12px, 2vw, 16px);
    border-radius: 12px;
    background: var(--surface-muted);
    border: 1px dashed var(--border);
    color: var(--muted);
  }
</style>
