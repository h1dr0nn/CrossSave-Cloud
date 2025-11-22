<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import type { HistoryEntry } from "../lib/api";

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
    class="backdrop"
    role="button"
    tabindex="0"
    aria-label="Close compare overlay"
    on:click={close}
    on:keydown={handleKey}
    on:keypress={handleKey}
  >
    <div
      class="drawer"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      on:click|stopPropagation
      on:keydown|stopPropagation
    >
      <header>
        <div>
          <p class="eyebrow">Compare Versions</p>
          <h2>
            {primary.metadata.version_id} vs {secondary?.metadata.version_id ??
              "—"}
          </h2>
        </div>
        <button class="close" on:click={close} aria-label="Close compare"
          >×</button
        >
      </header>

      <div class="grid">
        <div class="metric">
          <p class="label">Hash</p>
          <div class="compare">
            <span
              class={diffClass(primary.metadata.hash, secondary?.metadata.hash)}
            >
              {shortHash(primary.metadata.hash)}
            </span>
            <span class="divider">→</span>
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

        <div class="metric">
          <p class="label">File count</p>
          <div class="compare">
            <span class={diffClass(fileCount(primary), fileCount(secondary))}
              >{fileCount(primary)}</span
            >
            <span class="divider">→</span>
            <span class={diffClass(fileCount(secondary), fileCount(primary))}
              >{fileCount(secondary)}</span
            >
          </div>
        </div>

        <div class="metric">
          <p class="label">Total size</p>
          <div class="compare">
            <span class={diffClass(sizeValue(primary), sizeValue(secondary))}
              >{totalSize(primary)}</span
            >
            <span class="divider">→</span>
            <span class={diffClass(sizeValue(secondary), sizeValue(primary))}
              >{totalSize(secondary)}</span
            >
          </div>
        </div>
      </div>

      <p class="note">
        Comparing the two most recent versions near your selection.
      </p>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.3);
    display: grid;
    place-items: center;
    padding: 12px;
    z-index: 20;
    border: none;
    cursor: pointer;
  }

  .drawer {
    width: min(620px, 100%);
    background: color-mix(in srgb, var(--surface) 95%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    border-radius: 18px;
    padding: 16px 18px;
    padding-bottom: max(16px, env(safe-area-inset-bottom));
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.25);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  h2 {
    margin: 4px 0 0;
    font-size: 1.2rem;
  }

  .eyebrow {
    margin: 0;
    color: var(--muted);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    font-size: 0.8rem;
  }

  .close {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: 1px solid var(--border);
    background: var(--surface);
    cursor: pointer;
    font-size: 1.2rem;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 12px;
    margin: 14px 0;
  }

  .metric {
    border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
    border-radius: 14px;
    padding: 12px;
    background: color-mix(in srgb, var(--surface-muted) 60%, var(--surface));
  }

  .label {
    margin: 0 0 6px;
    color: var(--muted);
  }

  .compare {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: "SFMono-Regular", ui-monospace, SFMono-Regular, Menlo, Monaco,
      Consolas, "Liberation Mono", "Courier New", monospace;
  }

  .divider {
    color: var(--muted);
  }

  .positive {
    color: #16a34a;
  }

  .negative {
    color: #dc2626;
  }

  .neutral {
    color: var(--text);
  }

  .note {
    margin: 0;
    color: var(--muted);
    font-size: 0.95rem;
  }
</style>
