<script lang="ts">
  import { onMount } from "svelte";

  import CompareVersionDrawer from "./CompareVersionDrawer.svelte";
  import RecentHistory from "./RecentHistory.svelte";
  import { listHistory, packageGame, type HistoryEntry } from "../lib/api";
  import { pushError, pushInfo } from "../lib/notifications";
  import { goto } from "$app/navigation";

  export let gameId: string;

  let emulatorId = "";
  let gameName = "";
  let loading = true;
  let packaging = false;
  let reloading = false;
  let history: HistoryEntry[] = [];
  let drawerOpen = false;
  let selectedVersion: HistoryEntry | null = null;

  const formatter = new Intl.DateTimeFormat(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit"
  });

  onMount(() => {
    gameName = buildName(gameId);
    loadHistory();
  });

  $: latestEntry = history[0] ?? null;
  $: emulatorId = latestEntry?.metadata.emulator_id ?? emulatorId ?? deriveEmulatorId(gameId);
  $: lastModified = latestEntry ? formatter.format(latestEntry.metadata.timestamp) : "—";
  $: matchedFiles = latestEntry?.metadata.file_list.length ?? 0;
  $: latestHash = latestEntry ? shortHash(latestEntry.metadata.hash) : "—";

  function buildName(id: string) {
    if (!id) return "Unknown Game";
    const clean = id.replace(/[_-]+/g, " ").trim();
    return clean
      .split(" ")
      .map((chunk) => chunk.charAt(0).toUpperCase() + chunk.slice(1))
      .join(" ");
  }

  function deriveEmulatorId(id: string) {
    const segments = id.split("-");
    return segments[0] || "";
  }

  function shortHash(hash: string) {
    return hash?.slice(0, 8) ?? "";
  }

  function goBack() {
    drawerOpen = false;
    goto("/", { keepFocus: true, noScroll: true });
  }

  async function loadHistory() {
    try {
      if (loading) {
        history = [];
      }
      reloading = true;
      const entries = await listHistory(gameId);
      history = [...entries].sort(
        (a, b) => b.metadata.timestamp - a.metadata.timestamp
      );
    } catch (error) {
      pushError(`Failed to load history: ${error}`);
    } finally {
      loading = false;
      reloading = false;
    }
  }

  async function packageNow() {
    if (!emulatorId) {
      emulatorId = deriveEmulatorId(gameId);
    }

    if (!emulatorId) {
      pushError("Missing emulator id for packaging");
      return;
    }

    packaging = true;
    try {
      await packageGame(emulatorId, gameId);
      pushInfo("Packaging completed");
      await loadHistory();
    } catch (error) {
      pushError(`Packaging failed: ${error}`);
    } finally {
      packaging = false;
    }
  }

  function handleSelect(event: CustomEvent<HistoryEntry>) {
    selectedVersion = event.detail;
    drawerOpen = true;
  }

  function closeDrawer() {
    drawerOpen = false;
  }
</script>

<section class="detail-shell">
  <header class="detail-header">
    <button class="icon-button" on:click={goBack} aria-label="Go back">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M14.5 6 8.5 12l6 6"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
        />
      </svg>
    </button>

    <div class="header-actions">
      <button class="primary" on:click={packageNow} disabled={packaging}>
        {packaging ? "Packaging..." : "Package now"}
      </button>
      <button class="ghost" on:click={loadHistory} disabled={reloading}>
        {reloading ? "Refreshing" : "Reload"}
      </button>
    </div>
  </header>

  <div class="hero">
    <div class="icon" aria-hidden="true">{gameName.charAt(0) || "G"}</div>
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
  </div>

  <div class="panels">
    <article class="info-card summary">
      <div class="summary-header">
        <p class="label">Latest version</p>
        <span class="pill">{latestEntry?.metadata.version_id ?? "—"}</span>
      </div>
      <dl>
        <div>
          <dt>Emulator</dt>
          <dd>{emulatorId || "Not provided"}</dd>
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
      {history}
      on:select={handleSelect}
      on:reload={loadHistory}
      loading={loading}
    />
  </div>
</section>

<CompareVersionDrawer
  open={drawerOpen}
  on:close={closeDrawer}
  selected={selectedVersion}
  entries={history}
/>

<style>
  .detail-shell {
    max-width: 1200px;
    margin: 0 auto;
    padding: clamp(16px, 4vw, 32px);
    display: flex;
    flex-direction: column;
    gap: 18px;
    min-height: 100vh;
    color: var(--text);
  }

  .detail-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: var(--radius);
    background: color-mix(in srgb, var(--surface) 92%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 90%, transparent);
    box-shadow: var(--shadow-soft);
    min-width: 0;
  }

  .icon-button {
    width: 40px;
    height: 40px;
    border-radius: 14px;
    border: 1px solid var(--border);
    background: var(--surface);
    display: grid;
    place-items: center;
    color: var(--text);
    box-shadow: var(--shadow-soft);
    cursor: pointer;
    transition: transform 0.12s ease, box-shadow 0.18s ease, opacity 0.16s ease,
      border-color 0.18s ease;
  }

  .icon-button svg {
    width: 22px;
    height: 22px;
  }

  .icon-button:hover:not(:disabled),
  .ghost:hover:not(:disabled),
  .primary:hover:not(:disabled) {
    opacity: 0.92;
    border-color: var(--accent);
    box-shadow: var(--shadow);
  }

  .icon-button:active:not(:disabled),
  .ghost:active:not(:disabled),
  .primary:active:not(:disabled) {
    transform: scale(0.98);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: clamp(8px, 1vw, 12px);
    flex-wrap: nowrap;
    min-width: 0;
    margin-left: auto;
  }

  .header-actions button {
    white-space: nowrap;
    flex: 0 1 auto;
  }

  .hero {
    background: color-mix(in srgb, var(--surface) 90%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    border-radius: var(--radius);
    padding: clamp(16px, 2vw, 22px);
    display: grid;
    grid-template-columns: auto 1fr;
    gap: clamp(12px, 2vw, 18px);
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
    background: radial-gradient(circle at 10% 20%, color-mix(in srgb, var(--accent) 8%, transparent), transparent 35%),
      radial-gradient(circle at 90% 0%, color-mix(in srgb, var(--accent-strong) 10%, transparent), transparent 40%);
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
    transition: transform 0.12s ease, box-shadow 0.18s ease, opacity 0.16s ease,
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
    transition: transform 0.12s ease, box-shadow 0.18s ease, opacity 0.16s ease,
      border-color 0.18s ease;
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 14px;
    align-items: stretch;
  }

  .info-card {
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    background: color-mix(in srgb, var(--surface) 94%, transparent);
    border-radius: var(--radius);
    padding: 16px 18px;
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
    font-family: "SFMono-Regular", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas,
      "Liberation Mono", "Courier New", monospace;
  }

  .panels {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: 14px;
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
    gap: 12px;
    margin: 12px 0 0;
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
