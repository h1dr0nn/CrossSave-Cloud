<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import type { HistoryEntry } from "../../lib/api";

  export let open = false;
  export let selected: HistoryEntry | null = null;
  export let entries: HistoryEntry[] = [];

  const dispatch = createEventDispatcher<{ close: void }>();

  $: sorted = [...entries].sort(
    (a, b) => b.metadata.timestamp - a.metadata.timestamp
  );
  $: primary = selected ?? sorted[0] ?? null;
  $: secondary = determineSecondary(primary, sorted);

  function determineSecondary(
    primaryEntry: HistoryEntry | null,
    list: HistoryEntry[]
  ) {
    if (!primaryEntry || list.length < 2) return list[1] ?? null;
    const index = list.findIndex(
      (item) => item.metadata.version_id === primaryEntry.metadata.version_id
    );
    if (index === -1) return list[1] ?? null;
    return list[index + 1] ?? list[index - 1] ?? null;
  }

  function shortHash(hash: string | undefined) {
    return hash?.slice(0, 8) ?? "—";
  }

  function fileCount(entry: HistoryEntry | null) {
    return entry?.metadata.file_list.length ?? 0;
  }

  function sizeValue(entry: HistoryEntry | null) {
    return fileCount(entry) * 128 * 1024;
  }

  function totalSize(entry: HistoryEntry | null) {
    const bytes = sizeValue(entry);
    const units = ["B", "KB", "MB", "GB"];
    let value = bytes;
    let unitIndex = 0;

    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex += 1;
    }

    return `${value.toFixed(1)} ${units[unitIndex]}`;
  }

  function diffClass(a: number | string, b: number | string) {
    if (a === b) return "neutral";
    if (typeof a === "number" && typeof b === "number") {
      return a > b ? "positive" : "negative";
    }
    return "negative";
  }

  function handleKey(event: KeyboardEvent) {
    if (event.key === "Escape" || event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      close();
    }
  }

  function close() {
    dispatch("close");
  }

  $: if (typeof document !== "undefined") {
    if (open) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "";
    }
  }

  import { onDestroy } from "svelte";
  onDestroy(() => {
    if (typeof document !== "undefined") {
      document.body.style.overflow = "";
    }
  });
</script>

{#if open && primary}
  <div
    class="modal-backdrop"
    on:click={close}
    on:keydown={handleKey}
    role="button"
    tabindex="0"
    aria-label="Close compare overlay"
  >
    <div
      class="modal-content"
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <header class="modal-header">
        <h3>Compare Versions</h3>
        <button class="close-btn" on:click={close} aria-label="Close">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </header>

      <div class="modal-body">
        <div class="version-comparison">
          <span class="pill version-id" title={primary.metadata.version_id}>
            {primary.metadata.version_id}
          </span>
          <span class="vs">vs</span>
          <span
            class="pill version-id"
            title={secondary?.metadata.version_id ?? "—"}
          >
            {secondary?.metadata.version_id ?? "—"}
          </span>
        </div>

        <div class="metrics-grid">
          <div class="metric-card">
            <p class="metric-label">Hash</p>
            <div class="metric-values">
              <span
                class={diffClass(
                  primary.metadata.hash,
                  secondary?.metadata.hash
                )}
              >
                {shortHash(primary.metadata.hash)}
              </span>
              <span class="arrow">→</span>
              <span
                class={diffClass(
                  secondary?.metadata.hash ?? "",
                  primary.metadata.hash
                )}
              >
                {shortHash(secondary?.metadata.hash)}
              </span>
            </div>
          </div>

          <div class="metric-card">
            <p class="metric-label">File count</p>
            <div class="metric-values">
              <span class={diffClass(fileCount(primary), fileCount(secondary))}>
                {fileCount(primary)}
              </span>
              <span class="arrow">→</span>
              <span class={diffClass(fileCount(secondary), fileCount(primary))}>
                {fileCount(secondary)}
              </span>
            </div>
          </div>

          <div class="metric-card">
            <p class="metric-label">Total size</p>
            <div class="metric-values">
              <span class={diffClass(sizeValue(primary), sizeValue(secondary))}>
                {totalSize(primary)}
              </span>
              <span class="arrow">→</span>
              <span class={diffClass(sizeValue(secondary), sizeValue(primary))}>
                {totalSize(secondary)}
              </span>
            </div>
          </div>
        </div>

        <p class="note">
          Comparing the two most recent versions near your selection.
        </p>
      </div>
    </div>
  </div>
{/if}
```

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    padding: 16px;
  }

  .modal-content {
    background: rgb(var(--surface-rgb));
    border: 1px solid var(--border);
    border-radius: var(--radius);
    width: 100%;
    max-width: 720px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: var(--shadow-lg);
    animation: slideUp 0.2s ease-out;
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .modal-header {
    padding: 20px 24px;
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-radius: var(--radius) var(--radius) 0 0;
    background: rgb(var(--surface-rgb));
  }

  .modal-header h3 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text);
  }

  .close-btn {
    background: none;
    border: none;
    padding: 4px;
    cursor: pointer;
    color: var(--text-secondary);
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  .modal-body {
    padding: 24px;
    overflow-y: auto;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    gap: 20px;
    background: rgb(var(--surface-rgb));
    flex: 1 1 auto;
    min-height: 0;
    border-radius: 0 0 var(--radius) var(--radius);
  }

  .version-comparison {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 12px;
    background: color-mix(in srgb, var(--surface-muted) 50%, transparent);
    border-radius: var(--radius);
    flex-wrap: wrap;
  }

  .pill {
    padding: 6px 12px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--accent-muted) 60%, var(--surface));
    border: 1px solid color-mix(in srgb, var(--accent) 30%, var(--border));
    font-weight: 600;
    font-size: 0.85rem;
  }

  .version-id {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: "SFMono-Regular", ui-monospace, monospace;
  }

  .vs {
    color: var(--muted);
    font-weight: 500;
    font-size: 0.9rem;
  }

  @media (max-width: 600px) {
    .version-comparison {
      flex-direction: column;
    }
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
  }

  @media (max-width: 720px) {
    .metrics-grid {
      grid-template-columns: 1fr;
    }
  }

  .metric-card {
    border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
    border-radius: 12px;
    padding: 14px;
    background: color-mix(in srgb, var(--surface-muted) 40%, transparent);
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 160px;
  }

  .metric-label {
    margin: 0;
    color: var(--muted);
    font-size: 0.85rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .metric-values {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: "SFMono-Regular", ui-monospace, monospace;
    font-size: 0.95rem;
    justify-content: center;
  }

  .arrow {
    color: var(--muted);
    font-size: 1rem;
  }

  .positive {
    color: #16a34a;
    font-weight: 600;
  }

  .negative {
    color: #dc2626;
    font-weight: 600;
  }

  .neutral {
    color: var(--text);
    font-weight: 600;
  }

  .note {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
    text-align: center;
    padding: 12px;
    background: color-mix(in srgb, var(--surface-muted) 30%, transparent);
    border-radius: var(--radius-sm);
  }

  @media (max-width: 600px) {
    .modal-content {
      max-width: 100%;
      max-height: 95vh;
      border-radius: var(--radius-sm);
    }

    .modal-header,
    .modal-body {
      padding: 16px;
    }
  }
</style>
