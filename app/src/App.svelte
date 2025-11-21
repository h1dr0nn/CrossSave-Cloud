<script lang="ts">
  import MainLayout from "./components/MainLayout.svelte";
  import "./lib/themeStore";
</script>

<MainLayout />

<style>
  :global(body) {
    margin: 0;
    min-height: 100vh;
    font-family: "Inter", system-ui, -apple-system, "Segoe UI", sans-serif;
    background: var(--bg);
    color: var(--text);
    transition: background-color 0.2s ease, color 0.2s ease;
  }

  :global(body.theme-light) {
    --bg: radial-gradient(circle at 20% 20%, #f8fafc, #e2e8f0 45%, #cbd5e1 70%);
    --surface: #ffffff;
    --surface-muted: #f8fafc;
    --text: #0f172a;
    --muted: #475569;
    --border: #e2e8f0;
    --shadow-strong: 0 10px 35px rgba(15, 23, 42, 0.08);
    --shadow: 0 10px 30px rgba(15, 23, 42, 0.06);
    --overlay: rgba(15, 23, 42, 0.35);
    --accent: #38bdf8;
    --accent-strong: #0ea5e9;
    --accent-muted: #e0f2fe;
    --card-contrast: #0f172a;
    --input: #edf2f7;
  }

  :global(body.theme-dark) {
    --bg: radial-gradient(circle at 20% 20%, #0f172a, #0b1220 45%, #0b1324 70%);
    --surface: #0f1629;
    --surface-muted: #111b31;
    --text: #e2e8f0;
    --muted: #94a3b8;
    --border: #1f2937;
    --shadow-strong: 0 12px 32px rgba(0, 0, 0, 0.55);
    --shadow: 0 10px 24px rgba(0, 0, 0, 0.45);
    --overlay: rgba(0, 0, 0, 0.55);
    --accent: #38bdf8;
    --accent-strong: #0ea5e9;
    --accent-muted: #0b5071;
    --card-contrast: #0b1220;
    --input: #0b1220;
  }

  :global(a) {
    color: inherit;
    text-decoration: none;
  }

  :global(*), :global(*::before), :global(*::after) {
    box-sizing: border-box;
  }
</style>
