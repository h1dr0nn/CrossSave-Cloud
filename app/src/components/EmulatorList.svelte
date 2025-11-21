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
      </button>
    {/each}
  {/if}
</div>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: clamp(6px, 1vw, 10px);
    max-height: calc(100vh - clamp(110px, 14vw, 180px));
    overflow-y: auto;
    padding: clamp(12px, 2vw, 16px) clamp(2px, 0.6vw, 6px) clamp(2px, 0.6vw, 6px) 0;
  }

  button {
    position: relative;
    border: 1px solid var(--border);
    background: linear-gradient(180deg, color-mix(in srgb, var(--surface) 90%, transparent), var(--surface));
    border-radius: 12px;
    padding: clamp(10px, 2vw, 14px);
    display: grid;
    grid-template-columns: auto 1fr;
    gap: clamp(10px, 1.5vw, 14px);
    align-items: center;
    text-align: left;
    cursor: pointer;
    transition: border-color 0.2s ease, background 0.2s ease, transform 0.15s ease, box-shadow 0.2s ease;
    color: var(--text);
  }

  button::before {
    content: "";
    position: absolute;
    inset: 6px auto 6px -3px;
    width: 4px;
    border-radius: 8px;
    background: transparent;
    transition: background 0.2s ease;
  }

  button.selected {
    border-color: var(--accent);
    background: linear-gradient(120deg, color-mix(in srgb, var(--accent-muted) 80%, var(--surface) 20%), var(--surface));
    box-shadow: 0 10px 20px color-mix(in srgb, var(--accent-strong) 28%, transparent);
    transform: translateY(-1px);
  }

  button.selected::before {
    background: var(--accent);
  }

  button:hover {
    border-color: color-mix(in srgb, var(--accent) 60%, var(--border));
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
