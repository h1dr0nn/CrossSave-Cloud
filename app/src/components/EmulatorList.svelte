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
    padding-right: clamp(2px, 0.6vw, 6px);
  }

  button {
    border: 1px solid #e2e8f0;
    background: rgba(248, 250, 252, 0.8);
    border-radius: 12px;
    padding: clamp(10px, 2vw, 14px);
    display: grid;
    grid-template-columns: auto 1fr;
    gap: clamp(10px, 1.5vw, 14px);
    align-items: center;
    text-align: left;
    cursor: pointer;
    transition: border-color 0.2s ease, background 0.2s ease, transform 0.15s ease;
  }

  button.selected {
    border-color: #38bdf8;
    background: linear-gradient(120deg, #e0f2fe, #f8fafc);
    box-shadow: 0 10px 20px rgba(14, 165, 233, 0.2);
    transform: translateY(-1px);
  }

  button:hover {
    border-color: #cbd5e1;
  }

  .icon {
    width: clamp(36px, 5vw, 44px);
    height: clamp(36px, 5vw, 44px);
    border-radius: 12px;
    display: grid;
    place-items: center;
    color: #0f172a;
  }

  .icon svg {
    width: 100%;
    height: 100%;
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: clamp(4px, 0.8vw, 8px);
  }

  .text span {
    font-weight: 700;
    font-size: clamp(1rem, 0.5vw + 0.9rem, 1.05rem);
    color: #0f172a;
  }

  .text small {
    color: #475569;
    font-size: clamp(0.8rem, 0.3vw + 0.7rem, 0.95rem);
  }

  .empty {
    margin: 0;
    padding: clamp(12px, 2vw, 16px);
    border-radius: 12px;
    background: #f8fafc;
    border: 1px dashed #cbd5e1;
    color: #475569;
  }
</style>
