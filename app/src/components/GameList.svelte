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
    minute: "2-digit",
  });

  function toReadable(date: string) {
    return formatter.format(new Date(date));
  }

  function openGame(id: string) {
    goto(`/game/${encodeURIComponent(id)}`);
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
                <rect
                  x="4"
                  y="8"
                  width="16"
                  height="8"
                  rx="2"
                  fill="var(--card-contrast)"
                />
                <path d="M9.5 11h5v2h-5z" fill="#22c55e" />
                <circle cx="8" cy="12" r="1" fill="#22c55e" />
                <circle cx="16" cy="12" r="1" fill="#22c55e" />
              </svg>
            {:else if game.icon === "spark"}
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path
                  d="M12 3 9.5 11h-5L10 13l-2.5 8L12 15l4.5 6-2.5-8 5.5-2h-5Z"
                  fill="#0ea5e9"
                />
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
            <path
              d="m9 5 7 7-7 7"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
            />
          </svg>
        </button>
      {/each}
    </div>
  {/if}
</section>

<style>
  .panel {
    background: color-mix(in srgb, var(--surface) 94%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 88%, transparent);
    border-radius: var(--radius);
    padding: clamp(14px, 2vw, 18px);
    box-shadow: var(--shadow-soft);
    min-height: 0;
    color: var(--text);
    backdrop-filter: blur(12px) saturate(1.05);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
  }

  h2 {
    margin: 2px 0 0;
    font-size: clamp(1.1rem, 0.7vw + 1rem, 1.35rem);
    color: var(--text);
  }

  .eyebrow {
    margin: 0;
    color: var(--muted);
    font-size: 0.8rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .badge {
    padding: 8px 12px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--accent-muted) 70%, var(--surface));
    color: var(--accent-strong);
    font-weight: 700;
    font-size: 0.9rem;
    border: 1px solid color-mix(in srgb, var(--accent) 35%, var(--border));
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.3);
  }

  .empty {
    margin: 0;
    padding: 14px;
    background: var(--surface-muted);
    border-radius: var(--radius-sm);
    border: 1px dashed var(--border);
    color: var(--muted);
  }

  .grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .card {
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    border-radius: 16px;
    padding: 14px 16px;
    background: linear-gradient(
      180deg,
      color-mix(in srgb, var(--surface-muted) 92%, transparent),
      var(--surface)
    );
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 12px;
    text-align: left;
    cursor: pointer;
    transition:
      transform 0.18s ease,
      box-shadow 0.18s ease,
      border-color 0.18s ease,
      background 0.2s ease;
    color: var(--text);
    box-shadow: 0 8px 18px color-mix(in srgb, var(--shadow) 30%, transparent);
  }

  .card:hover {
    transform: translateY(-2px);
    border-color: var(--accent);
    box-shadow: 0 16px 30px
      color-mix(in srgb, var(--accent-strong) 22%, transparent);
  }

  .icon {
    width: 52px;
    height: 52px;
    border-radius: 16px;
    display: grid;
    place-items: center;
    background: var(--card-contrast);
    color: var(--text);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.15);
  }

  .icon[data-variant="spark"] {
    background: linear-gradient(135deg, var(--accent), var(--accent-strong));
  }

  .icon[data-variant="disc"] {
    background: linear-gradient(135deg, #a855f7, #6366f1);
  }

  .meta {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .meta strong {
    font-size: 1rem;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .meta span {
    color: var(--muted);
    font-size: 0.9rem;
  }

  .chevron {
    width: 20px;
    height: 20px;
    color: var(--muted);
    stroke-width: 1.6;
  }

  @media (min-width: 680px) {
    .grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (min-width: 1180px) {
    .grid {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }
  }
</style>
