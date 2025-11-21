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
  <div class="page-header">
    <button class="icon-button" on:click={goBack} aria-label="Go back">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M14.5 6 8.5 12l6 6" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
      </svg>
    </button>
  </div>

  <div class="hero">
    <div class="icon" aria-hidden="true">{gameName.charAt(0) || "G"}</div>
    <div class="heading">
      <p class="eyebrow">Save Management</p>
      <h1>{gameName}</h1>
      <p class="meta">Game ID: {gameId}</p>
    </div>
    <div class="actions">
      <button class="primary" on:click={packageNow} disabled={packaging}>
        {packaging ? "Packaging..." : "Package now"}
      </button>
      <button class="ghost" on:click={loadHistory} disabled={reloading}>
        {reloading ? "Refreshing" : "Reload"}
      </button>
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

  .hero {
    background: color-mix(in srgb, var(--surface) 90%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    border-radius: var(--radius);
    padding: clamp(16px, 2vw, 22px);
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 14px;
    align-items: center;
    box-shadow: var(--shadow-soft);
  }

  .page-header {
    display: flex;
    align-items: center;
    justify-content: flex-start;
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
    transition: transform 0.15s ease, box-shadow 0.2s ease, border-color 0.2s ease;
  }

  .icon-button:hover {
    transform: translateY(-1px);
    border-color: var(--accent);
    box-shadow: var(--shadow);
  }

  .icon-button svg {
    width: 22px;
    height: 22px;
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
    font-size: clamp(1.2rem, 1vw + 1rem, 1.8rem);
  }

  .heading .meta {
    margin: 0;
    color: var(--muted);
  }

  .eyebrow {
    margin: 0;
    color: var(--muted);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    font-size: 0.8rem;
  }

  .actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .primary {
    padding: 10px 14px;
    border-radius: 12px;
    background: var(--accent);
    color: #fff;
    border: 1px solid color-mix(in srgb, var(--accent-strong) 70%, transparent);
    box-shadow: var(--shadow);
    cursor: pointer;
    min-width: 140px;
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
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 12px;
  }

  .info-card {
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    background: color-mix(in srgb, var(--surface) 90%, transparent);
    border-radius: var(--radius);
    padding: 14px 16px;
    box-shadow: var(--shadow-soft);
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
    grid-template-columns: 1fr;
    gap: 12px;
  }

  @media (min-width: 960px) {
    .panels {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 14px;
    }
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
      grid-template-columns: 1fr;
    }

    .actions {
      justify-content: flex-start;
    }
  }
</style>
