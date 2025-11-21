<script lang="ts">
  import MainLayout from "./components/MainLayout.svelte";
</script>

<MainLayout />

<style>
  :global(body) {
    margin: 0;
    min-height: 100vh;
    font-family: "Inter", system-ui, -apple-system, "Segoe UI", sans-serif;
    background: radial-gradient(circle at 20% 20%, #f8fafc, #e2e8f0 45%, #cbd5e1 70%);
    color: #0f172a;
  }
</style>
