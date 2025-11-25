<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { get } from "svelte/store";
  import AppHeader from "../layout/AppHeader.svelte";
  import GameIcon from "./GameIcon.svelte";
  import CompareVersionDrawer from "./CompareVersionDrawer.svelte";
  import RecentHistory from "./RecentHistory.svelte";
  import type { HistoryEntry } from "../../lib/api";
  import {
    extractGameName,
    getIconVariant,
    deriveEmulatorId,
  } from "../../lib/utils";
  import { goto } from "$app/navigation";
  import { createGameDetailLogic } from "../../lib/logic/gameDetail";

  export let gameId: string;

  const logic = createGameDetailLogic(gameId);
  const {
    history,
    loading,
    reloading,
    packaging,
    watcherEvents,
    changesDetected,
    autoPackageEnabled,
    trackedPatterns,
    latestEntry,
    emulatorId: emulatorIdStore,
    loadHistory,
    packageNow,
    hydrateAutoPackage,
    startWatcherFeed,
    cleanup,
    setEmulatorId,
  } = logic;

  let drawerOpen = false;
  let selectedVersion: HistoryEntry | null = null;

  const timeFormatter = new Intl.DateTimeFormat(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });

  const formatter = new Intl.DateTimeFormat(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });

  onMount(() => {
    console.log(`[GameDetail] Mounted for gameId: ${gameId}`);

    // Ensure emulatorId is set immediately
    const initialEmulatorId = deriveEmulatorId(gameId);
    console.log(
      `[GameDetail] Initial emulatorId from derive: ${initialEmulatorId}`
    );
    if (initialEmulatorId && !get(emulatorIdStore)) {
      setEmulatorId(initialEmulatorId);
    }

    loadHistory();
    hydrateAutoPackage();
    startWatcherFeed();
  });

  onDestroy(() => {
    cleanup();
  });

  // Reactive derived state - ensure it's never empty
  $: effectiveEmulatorId =
    $latestEntry?.metadata.emulator_id ||
    $emulatorIdStore ||
    deriveEmulatorId(gameId) ||
    "retroarch"; // Fallback to retroarch

  $: {
    console.log(
      `[GameDetail] effectiveEmulatorId changed to: "${effectiveEmulatorId}"`
    );
    console.log(
      `[GameDetail] - from latestEntry: ${$latestEntry?.metadata.emulator_id ?? "null"}`
    );
    console.log(`[GameDetail] - from store: ${$emulatorIdStore ?? "null"}`);
    console.log(`[GameDetail] - from derive: ${deriveEmulatorId(gameId)}`);
  }

  $: if (effectiveEmulatorId && effectiveEmulatorId !== $emulatorIdStore) {
    console.log(`[GameDetail] Setting emulatorId to: ${effectiveEmulatorId}`);
    setEmulatorId(effectiveEmulatorId);
  }

  $: lastModified = $latestEntry
    ? formatter.format($latestEntry.metadata.timestamp)
    : "—";
  $: matchedFiles = $latestEntry?.metadata.file_list.length ?? 0;
  $: latestHash = $latestEntry ? shortHash($latestEntry.metadata.hash) : "—";

  $: storageUsed = (() => {
    if ($history.length === 0) return "—";
    // Estimate: avg 128KB per file
    const totalFiles = $history.reduce(
      (sum, entry) => sum + entry.metadata.file_list.length,
      0
    );
    const estimatedBytes = totalFiles * 128 * 1024;
    return formatBytes(estimatedBytes);
  })();

  $: gameName = extractGameName(gameId);
  $: iconVariant = getIconVariant(gameId);

  function shortHash(hash: string) {
    return hash?.slice(0, 8) ?? "";
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const units = ["B", "KB", "MB", "GB"];
    let value = bytes;
    let unitIndex = 0;
    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex++;
    }
    return `${value.toFixed(value >= 10 ? 0 : 1)} ${units[unitIndex]}`;
  }

  function goBack() {
    drawerOpen = false;
    goto("/", { keepFocus: true, noScroll: true });
  }

  function handleSelect(event: CustomEvent<HistoryEntry>) {
    selectedVersion = event.detail;
    drawerOpen = true;
  }

  function closeDrawer() {
    drawerOpen = false;
  }

  function handlePackage() {
    // Extract game name from file path
    // gameId might be a full path like "/path/to/game.srm"
    // We need just the basename without extension
    const gameBasename = gameId.split(/[/\\]/).pop() || gameId;
    const gameName = gameBasename.replace(/\.[^/.]+$/, ""); // Remove extension

    console.log(
      `[GameDetail] Packaging: gameId="${gameId}", extracted name="${gameName}"`
    );
    packageNow(effectiveEmulatorId, gameName);
  }
</script>

<section class="detail-shell">
  <AppHeader showBack onBack={goBack} onMenu={() => {}} sticky={false}>
    <div slot="actions" class="header-actions">
      {#if $changesDetected}
        <span class="pill attention" aria-live="polite">Changes detected</span>
      {/if}
      <button
        class="icon-button"
        class:spinning={$reloading}
        on:click={loadHistory}
        disabled={$reloading}
        title="Reload history"
      >
        <svg
          viewBox="0 0 24 24"
          aria-hidden="true"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M23 4v6h-6" />
          <path d="M1 20v-6h6" />
          <path
            d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"
          />
        </svg>
      </button>
      <button class="primary" on:click={handlePackage} disabled={$packaging}>
        {$packaging ? "Packaging..." : "Package now"}
      </button>
    </div>
  </AppHeader>

  <div class="hero">
    <GameIcon name={gameName} id={gameId} variant="spark" size={64} />
    <div class="heading">
      <p class="section-title">Save Management</p>
      <h1>{gameName}</h1>
      <p class="meta">ID: {gameId}</p>
    </div>
  </div>

  <div class="info-grid">
    <article class="info-card">
      <p class="label">Last modified</p>
      <h3>{lastModified}</h3>
      <p class="hint">Latest update pulled from history</p>
    </article>
    <article class="info-card">
      <p class="label">Matched files</p>
      <h3>{matchedFiles}</h3>
      <p class="hint">Total files packaged in the newest version</p>
    </article>
    <article class="info-card">
      <p class="label">Latest hash</p>
      <h3 class="mono">{latestHash}</h3>
      <p class="hint">Version fingerprint (8 chars)</p>
    </article>
    <article class="info-card">
      <p class="label">Storage used</p>
      <h3>{storageUsed}</h3>
      <p class="hint">Estimated total size (all versions)</p>
    </article>
  </div>

  <div class="panels">
    <article class="info-card summary">
      <div class="summary-header">
        <p class="label">Latest version</p>
        <span class="pill">{$latestEntry?.metadata.version_id ?? "—"}</span>
      </div>
      <dl>
        <div>
          <dt>Emulator</dt>
          <dd>{effectiveEmulatorId || "Not provided"}</dd>
        </div>
        <div>
          <dt>Last updated</dt>
          <dd>{lastModified}</dd>
        </div>
        <div>
          <dt>Files tracked</dt>
          <dd>{matchedFiles}</dd>
        </div>
        <div>
          <dt>Hash</dt>
          <dd class="mono">{latestHash}</dd>
        </div>
      </dl>
    </article>
    <RecentHistory
      {gameId}
      history={$history}
      on:select={handleSelect}
      on:reload={loadHistory}
      loading={$loading}
    />
  </div>

  <section class="watcher-section">
    <div class="section-heading">
      <div>
        <p class="section-title">Watcher</p>
        <h2>Live save changes</h2>
        <p class="meta">
          Monitoring file events for <span class="mono"
            >{effectiveEmulatorId || "?"}</span
          >
          {#if $trackedPatterns.length}
            • {$trackedPatterns.length} pattern{$trackedPatterns.length === 1
              ? ""
              : "s"}
          {/if}
        </p>
      </div>

      <div class="watcher-actions">
        {#if $changesDetected}
          <span class="pill attention" aria-live="polite">Changes detected</span
          >
        {/if}
        <label class="toggle" aria-label="Auto-package on change">
          <input type="checkbox" bind:checked={$autoPackageEnabled} />
          <span class="track"></span>
          <span class="thumb"></span>
          <span class="toggle-label">Auto-package</span>
        </label>
      </div>
    </div>

    <article class="info-card watcher-card">
      <div class="feed-header">
        <div>
          <p class="label">Live feed</p>
          <p class="hint">Latest events appear first</p>
        </div>
        <span class="pill subtle">{$watcherEvents.length} events</span>
      </div>

      {#if $watcherEvents.length === 0}
        <p class="placeholder">Waiting for file change events...</p>
      {:else}
        <ul class="feed-list">
          {#each $watcherEvents as event (event.timestamp.toISOString() + event.path + event.kind)}
            <li class="feed-item">
              <div class={`dot ${event.kind}`}></div>
              <div class="feed-meta">
                <div class="row">
                  <span class="event-kind">{event.kind}</span>
                  <span class="timestamp"
                    >{timeFormatter.format(event.timestamp)}</span
                  >
                </div>
                <p class="file-name" title={event.path}>{event.fileName}</p>
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </article>
  </section>
</section>

<CompareVersionDrawer
  open={drawerOpen}
  on:close={closeDrawer}
  selected={selectedVersion}
  entries={$history}
/>

<style>
  .detail-shell {
    --space-sm: 12px;
    --space-md: 16px;
    --space-lg: 24px;
    max-width: 1200px;
    margin: 0 auto;
    padding: clamp(16px, 3vw, 32px);
    padding-top: max(clamp(16px, 3vw, 32px), env(safe-area-inset-top));
    padding-bottom: max(clamp(16px, 3vw, 32px), env(safe-area-inset-bottom));
    padding-left: max(clamp(16px, 3vw, 32px), env(safe-area-inset-left));
    padding-right: max(clamp(16px, 3vw, 32px), env(safe-area-inset-right));
    display: flex;
    flex-direction: column;
    gap: clamp(16px, 3vw, 32px);
    min-height: 100vh;
    color: var(--text);
  }

  .ghost:hover:not(:disabled),
  .primary:hover:not(:disabled) {
    opacity: 0.92;
    border-color: var(--accent);
    box-shadow: var(--shadow);
  }

  .ghost:active:not(:disabled),
  .primary:active:not(:disabled) {
    transform: scale(0.98);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: clamp(10px, 1vw, var(--space-md));
    flex-wrap: nowrap;
    min-width: 0;
    margin-left: auto;
  }

  .header-actions button {
    white-space: nowrap;
    flex: 0 1 auto;
  }

  .icon-button {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--muted);
    width: 36px;
    height: 36px;
    border-radius: 10px;
    display: grid;
    place-items: center;
    cursor: pointer;
    transition: all 0.2s;
  }

  .icon-button:hover:not(:disabled) {
    color: var(--text);
    background: var(--surface-muted);
  }

  .icon-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .icon-button svg {
    width: 18px;
    height: 18px;
  }

  .spinning svg {
    animation: spin 1s linear infinite;
    transform-origin: center;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .hero {
    background: color-mix(in srgb, var(--surface) 90%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    border-radius: var(--radius);
    padding: clamp(var(--space-md), 2vw, var(--space-lg));
    display: grid;
    grid-template-columns: auto 1fr;
    gap: clamp(var(--space-md), 2vw, var(--space-lg));
    align-items: center;
    box-shadow: var(--shadow-soft);
    backdrop-filter: blur(14px);
    position: relative;
    overflow: hidden;
  }

  .hero::after {
    content: "";
    position: absolute;
    inset: 0;
    background: radial-gradient(
        circle at 10% 20%,
        color-mix(in srgb, var(--accent) 8%, transparent),
        transparent 35%
      ),
      radial-gradient(
        circle at 90% 0%,
        color-mix(in srgb, var(--accent-strong) 10%, transparent),
        transparent 40%
      );
    pointer-events: none;
    opacity: 0.8;
  }

  .hero > * {
    position: relative;
    z-index: 1;
  }

  .icon {
    width: 56px;
    height: 56px;
    border-radius: 16px;
    background: linear-gradient(135deg, var(--accent), var(--accent-strong));
    color: #fff;
    font-weight: 800;
    display: grid;
    place-items: center;
    font-size: 1.2rem;
  }

  .heading h1 {
    margin: 4px 0 4px;
    font-size: clamp(1.7rem, 0.9vw + 1.3rem, 2.2rem);
    letter-spacing: -0.02em;
    font-weight: 800;
  }

  .section-title {
    margin: 0;
    color: var(--muted);
    letter-spacing: 0.14em;
    text-transform: uppercase;
    font-size: 0.78rem;
    font-weight: 700;
  }

  .meta {
    margin: 0;
    color: var(--muted);
    font-size: 0.95rem;
  }

  .primary {
    padding: 10px 14px;
    border-radius: 12px;
    background: var(--accent);
    color: #fff;
    border: 1px solid color-mix(in srgb, var(--accent-strong) 70%, transparent);
    box-shadow: var(--shadow);
    cursor: pointer;
    min-width: clamp(120px, 14vw, 150px);
    transition:
      transform 0.12s ease,
      box-shadow 0.18s ease,
      opacity 0.16s ease,
      border-color 0.18s ease;
  }

  .primary:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .ghost {
    padding: 10px 14px;
    border-radius: 12px;
    background: var(--surface-muted);
    color: var(--text);
    border: 1px solid var(--border);
    cursor: pointer;
    transition:
      transform 0.12s ease,
      box-shadow 0.18s ease,
      opacity 0.16s ease,
      border-color 0.18s ease;
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-lg);
    align-items: stretch;
  }

  @media (max-width: 1100px) {
    .info-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (max-width: 600px) {
    .info-grid {
      grid-template-columns: 1fr;
    }
  }

  .info-card {
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    background: color-mix(in srgb, var(--surface) 94%, transparent);
    border-radius: var(--radius);
    padding: var(--space-md);
    box-shadow: var(--shadow-soft);
    backdrop-filter: blur(12px);
  }

  .label {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  h3 {
    margin: 4px 0 6px;
    font-size: 1.3rem;
  }

  .hint {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .mono {
    font-family: "SFMono-Regular", ui-monospace, SFMono-Regular, Menlo, Monaco,
      Consolas, "Liberation Mono", "Courier New", monospace;
  }

  .panels {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: var(--space-lg);
    align-items: stretch;
  }

  .summary-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  dl {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: var(--space-md);
    margin: var(--space-sm) 0 0;
  }

  dt {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  dd {
    margin: 2px 0 0;
    font-weight: 700;
  }

  .pill {
    padding: 6px 10px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--accent-muted) 60%, var(--surface));
    border: 1px solid color-mix(in srgb, var(--accent) 30%, var(--border));
    font-weight: 700;
  }

  .summary {
    min-height: 100%;
  }

  .watcher-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .section-heading {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--space-md);
    flex-wrap: wrap;
  }

  .watcher-actions {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .watcher-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .feed-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-md);
    flex-wrap: wrap;
  }

  .feed-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
    max-height: 360px;
    overflow-y: auto;
  }

  .feed-item {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 10px;
    align-items: start;
    padding: 8px;
    border-radius: 12px;
    background: color-mix(in srgb, var(--surface-muted) 70%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
  }

  .feed-meta {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .event-kind {
    text-transform: capitalize;
    font-weight: 800;
    color: var(--text);
  }

  .timestamp {
    color: var(--muted);
    font-size: 0.9rem;
  }

  .file-name {
    margin: 0;
    font-weight: 700;
    word-break: break-word;
  }

  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    margin-top: 4px;
    background: var(--muted);
    box-shadow: 0 0 0 6px color-mix(in srgb, var(--border) 60%, transparent);
  }

  .dot.create {
    background: #22c55e;
  }

  .dot.modify {
    background: #fb923c;
  }

  .dot.delete {
    background: #ef4444;
  }

  .pill.attention {
    background: color-mix(in srgb, var(--accent-muted) 50%, var(--surface));
    border-color: color-mix(in srgb, var(--accent) 60%, var(--border));
    color: var(--accent-strong);
    box-shadow: var(--shadow-soft);
  }

  .pill.subtle {
    background: color-mix(in srgb, var(--surface-muted) 80%, transparent);
    color: var(--muted);
    border-color: color-mix(in srgb, var(--border) 80%, transparent);
    font-weight: 600;
  }

  .toggle {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    font-weight: 700;
  }

  .toggle input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .toggle .track {
    width: 54px;
    height: 30px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--border) 80%, transparent);
    transition: background 0.2s ease;
    position: relative;
  }

  .toggle .thumb {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--surface);
    position: absolute;
    top: 4px;
    left: 4px;
    box-shadow: var(--shadow-soft);
    transition: transform 0.2s ease;
  }

  .toggle input:checked + .track {
    background: color-mix(in srgb, var(--accent) 50%, var(--accent-muted));
  }

  .toggle input:checked + .track + .thumb {
    transform: translateX(22px);
  }

  .toggle-label {
    color: var(--text);
    white-space: nowrap;
  }

  .placeholder {
    margin: 0;
    color: var(--muted);
  }

  @media (max-width: 720px) {
    .hero {
      grid-template-columns: auto 1fr;
      align-items: start;
    }

    .header-actions {
      justify-content: flex-end;
      width: 100%;
    }
  }
</style>
