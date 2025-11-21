<script lang="ts">
  import { goto } from "$app/navigation";

  import type { GameEntry } from "../lib/uiTypes";

  export let games: GameEntry[] = [];
  export let emulatorName = "";

  const formatter = new Intl.DateTimeFormat(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit"
  });

  function toReadable(date: string) {
    return formatter.format(new Date(date));
  }

  function openGame(id: string) {
    goto(`/game/${id}`);
  }
</script>

<section class="panel">
  <header>
    <div>
      <p class="eyebrow">Games</p>
      <h2>{emulatorName || "Choose an emulator"}</h2>
    </div>
    <span class="badge">{games.length} titles</span>
  </header>

  {#if games.length === 0}
    <p class="empty">Select an emulator to view its saves.</p>
  {:else}
    <div class="grid">
      {#each games as game}
        <button class="card" on:click={() => openGame(game.id)}>
          <div class="icon" data-variant={game.icon}>
            {#if game.icon === "console"}
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <rect x="4" y="8" width="16" height="8" rx="2" fill="#0f172a" />
                <path d="M9.5 11h5v2h-5z" fill="#22c55e" />
                <circle cx="8" cy="12" r="1" fill="#22c55e" />
                <circle cx="16" cy="12" r="1" fill="#22c55e" />
              </svg>
            {:else if game.icon === "spark"}
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d="M12 3 9.5 11h-5L10 13l-2.5 8L12 15l4.5 6-2.5-8 5.5-2h-5Z" fill="#0ea5e9" />
              </svg>
            {:else}
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <circle cx="12" cy="12" r="8" fill="#6366f1" />
                <circle cx="12" cy="12" r="3" fill="#e0f2fe" />
              </svg>
            {/if}
          </div>
          <div class="meta">
            <strong>{game.name}</strong>
            <span>{toReadable(game.lastModified)}</span>
          </div>
          <svg class="chevron" viewBox="0 0 24 24" aria-hidden="true">
            <path d="m9 5 7 7-7 7" fill="none" stroke="#0f172a" stroke-width="2" stroke-linecap="round" />
          </svg>
        </button>
      {/each}
    </div>
  {/if}
</section>

<style>
  .panel {
    background: rgba(255, 255, 255, 0.9);
    border: 1px solid #e2e8f0;
    border-radius: clamp(12px, 1vw, 16px);
    padding: clamp(14px, 2vw, 20px);
    box-shadow: 0 10px 30px rgba(15, 23, 42, 0.08);
    min-height: 0;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: clamp(12px, 2vw, 16px);
  }

  h2 {
    margin: 2px 0 0;
    font-size: clamp(1.2rem, 0.8vw + 1rem, 1.5rem);
    color: #0f172a;
  }

  .eyebrow {
    margin: 0;
    color: #475569;
    font-size: clamp(0.85rem, 0.3vw + 0.75rem, 0.95rem);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .badge {
    padding: clamp(6px, 1vw, 10px) clamp(10px, 1.4vw, 14px);
    border-radius: 999px;
    background: #e0f2fe;
    color: #0ea5e9;
    font-weight: 700;
    font-size: clamp(0.85rem, 0.3vw + 0.7rem, 0.95rem);
  }

  .empty {
    margin: 0;
    padding: clamp(12px, 2vw, 16px);
    background: #f8fafc;
    border-radius: 12px;
    border: 1px dashed #cbd5e1;
    color: #475569;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(clamp(160px, 32vw, 220px), 1fr));
    gap: clamp(10px, 1.2vw, 16px);
  }

  .card {
    border: 1px solid #e2e8f0;
    border-radius: clamp(12px, 1vw, 16px);
    padding: clamp(12px, 1.5vw, 16px);
    background: linear-gradient(180deg, rgba(248, 250, 252, 0.9), #fff);
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: clamp(10px, 1vw, 14px);
    text-align: left;
    cursor: pointer;
    transition: transform 0.18s ease, box-shadow 0.18s ease, border-color 0.18s ease;
  }

  .card:hover {
    transform: translateY(-2px);
    border-color: #38bdf8;
    box-shadow: 0 12px 24px rgba(14, 165, 233, 0.18);
  }

  .icon {
    width: clamp(44px, 6vw, 54px);
    height: clamp(44px, 6vw, 54px);
    border-radius: clamp(12px, 1vw, 16px);
    display: grid;
    place-items: center;
    background: #0f172a;
    color: #0f172a;
  }

  .icon[data-variant="spark"] {
    background: linear-gradient(135deg, #38bdf8, #0ea5e9);
  }

  .icon[data-variant="disc"] {
    background: linear-gradient(135deg, #a855f7, #6366f1);
  }

  .meta {
    display: flex;
    flex-direction: column;
    gap: clamp(4px, 0.8vw, 8px);
    min-width: 0;
  }

  .meta strong {
    font-size: clamp(1rem, 0.4vw + 0.9rem, 1.05rem);
    color: #0f172a;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .meta span {
    color: #475569;
    font-size: clamp(0.85rem, 0.3vw + 0.75rem, 0.95rem);
  }

  .chevron {
    width: clamp(18px, 3vw, 22px);
    height: clamp(18px, 3vw, 22px);
  }
</style>
