<script lang="ts">
  import WatcherPanel from "./components/WatcherPanel.svelte";
  import PackagerPanel from "./components/PackagerPanel.svelte";
  import HistoryPanel from "./components/HistoryPanel.svelte";

  type Tab = "watcher" | "packager" | "history";

  const tabs: { id: Tab; label: string }[] = [
    { id: "watcher", label: "Watcher" },
    { id: "packager", label: "Packager" },
    { id: "history", label: "History" }
  ];

  let activeTab: Tab = "watcher";
</script>

<main class="app">
  <header>
    <h1>CrossSave Cloud - Phase 1 Test UI</h1>
    <p>Quick controls to exercise backend watcher, packager, and history flows.</p>
  </header>

  <section class="tabs">
    {#each tabs as tab}
      <button class:active={activeTab === tab.id} on:click={() => (activeTab = tab.id)}>
        {tab.label}
      </button>
    {/each}
  </section>

  <section class="panel">
    {#if activeTab === "watcher"}
      <WatcherPanel />
    {:else if activeTab === "packager"}
      <PackagerPanel />
    {:else}
      <HistoryPanel />
    {/if}
  </section>
</main>

<style>
  .app {
    max-width: 960px;
    margin: 0 auto;
    padding: 24px 16px 48px;
    font-family: "Inter", system-ui, -apple-system, sans-serif;
    color: #0f172a;
  }

  header {
    margin-bottom: 16px;
  }

  h1 {
    margin: 0 0 4px 0;
    font-size: 24px;
  }

  p {
    margin: 0;
    font-size: 14px;
    color: #475569;
  }

  .tabs {
    display: flex;
    gap: 8px;
    margin: 16px 0;
  }

  button {
    padding: 8px 12px;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    background: #ffffff;
    cursor: pointer;
  }

  button.active {
    background: #e2e8f0;
    border-color: #94a3b8;
  }

  button:hover {
    background: #f8fafc;
  }

  .panel {
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    padding: 16px;
    background: #ffffff;
  }
</style>
