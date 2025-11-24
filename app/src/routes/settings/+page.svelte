<script lang="ts">
  import { onMount } from "svelte";
  import SettingsPage from "../../components/pages/SettingsPage.svelte";
  import CloudModeTab from "../../components/settings/CloudModeTab.svelte";

  const tabs = [
    { id: "general", label: "General" },
    { id: "cloud", label: "Cloud Mode" },
  ];

  let activeTab: "general" | "cloud" = "general";

  onMount(() => {
    activeTab = "general";
  });
</script>

<section class="tabbed-settings">
  <div class="tab-switcher" role="tablist" aria-label="Settings tabs">
    {#each tabs as tab}
      <button
        type="button"
        role="tab"
        aria-pressed={activeTab === tab.id}
        class:active={activeTab === tab.id}
        on:click={() => (activeTab = tab.id)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  {#if activeTab === "general"}
    <SettingsPage />
  {:else}
    <CloudModeTab />
  {/if}
</section>

<style>
  .tabbed-settings {
    min-height: 100vh;
    display: grid;
    gap: 16px;
    background: var(--bg);
  }

  .tab-switcher {
    margin: 0 auto;
    width: min(600px, 100%);
    padding: 16px clamp(16px, 4vw, 24px) 0;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 6px;
    background: var(--bg);
  }

  .tab-switcher button {
    border: 1px solid var(--border);
    background: color-mix(in srgb, var(--surface) 90%, transparent);
    padding: 10px 12px;
    border-radius: 12px;
    font-weight: 700;
    cursor: pointer;
    color: var(--text);
    transition: background 0.2s ease, color 0.2s ease;
  }

  .tab-switcher button.active,
  .tab-switcher button[aria-pressed="true"] {
    background: linear-gradient(135deg, #4f46e5, #7c3aed);
    color: white;
    box-shadow: 0 8px 18px rgba(79, 70, 229, 0.25);
  }

  @media (max-width: 640px) {
    .tab-switcher {
      padding-left: max(16px, env(safe-area-inset-left));
      padding-right: max(16px, env(safe-area-inset-right));
    }
  }
</style>
