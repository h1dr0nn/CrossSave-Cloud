<script lang="ts">
import { goto } from "$app/navigation";

import AppHeader from "../../components/AppHeader.svelte";
import ThemeSelector from "../../components/ThemeSelector.svelte";

function goBack() {
  goto("/", { keepFocus: true, noScroll: true });
}
</script>

<section class="settings">
  <AppHeader title="Settings" showBack onBack={goBack} onMenu={() => {}} />

  <div class="panel-stack">
    <ThemeSelector />
  </div>
</section>

<style>
  .settings {
    padding: clamp(16px, 3vw, 32px);
    display: grid;
    grid-template-rows: auto 1fr;
    gap: 18px;
    background: var(--bg);
    color: var(--text);
    min-height: 100vh;
  }

  .panel-stack {
    display: grid;
    gap: 18px;
    align-content: start;
  }
</style>
