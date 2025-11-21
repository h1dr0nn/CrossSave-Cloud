<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";

  import Sidebar from "./Sidebar.svelte";
  import GameList from "./GameList.svelte";
  import ProfilePreview from "./ProfilePreview.svelte";
  import type { GameEntry } from "../lib/uiTypes";
  import { listProfiles, type EmulatorProfile } from "../lib/api";

  let profiles: EmulatorProfile[] = [];
  let selectedEmulatorId: string | null = null;
  let viewportWidth = 960;
  let drawerOpen = false;
  let loadingProfiles = true;
  const gamesByEmulator = new Map<string, GameEntry[]>();

  onMount(async () => {
    try {
      const loadedProfiles = await listProfiles();
      profiles = loadedProfiles;
      if (loadedProfiles.length > 0) {
        selectedEmulatorId = loadedProfiles[0].emulator_id;
        preloadGames(loadedProfiles);
      }
    } catch (error) {
      console.error("Failed to load emulator profiles", error);
    } finally {
      loadingProfiles = false;
    }
  });

  $: selectedProfile = profiles.find((p) => p.emulator_id === selectedEmulatorId) ?? null;
  $: isMobile = viewportWidth < 600;
  $: selectedGames = selectedEmulatorId
    ? gamesByEmulator.get(selectedEmulatorId) ?? []
    : [];

  function preloadGames(items: EmulatorProfile[]) {
    items.forEach((profile) => {
      if (!gamesByEmulator.has(profile.emulator_id)) {
        gamesByEmulator.set(profile.emulator_id, generateGames(profile));
      }
    });
  }

  function generateGames(profile: EmulatorProfile): GameEntry[] {
    const baseName = profile.name.split(" ")[0] || profile.name;
    const now = Date.now();
    return Array.from({ length: 6 }, (_, index) => ({
      id: `${profile.emulator_id}-game-${index + 1}`,
      emulatorId: profile.emulator_id,
      name: `${baseName} Save ${index + 1}`,
      icon: index % 3 === 0 ? "spark" : index % 2 === 0 ? "disc" : "console",
      lastModified: new Date(now - (index + 1) * 36 * 60 * 60 * 1000).toISOString()
    }));
  }

  function selectEmulator(id: string) {
    selectedEmulatorId = id;
    if (isMobile) {
      drawerOpen = false;
    }
  }

  function toggleDrawer() {
    drawerOpen = !drawerOpen;
  }

  function openSettings() {
    goto("/settings");
  }
</script>

<svelte:window bind:innerWidth={viewportWidth} />

<div class="shell" data-mobile={isMobile}>
  <Sidebar
    emulators={profiles}
    selectedId={selectedEmulatorId}
    isMobile={isMobile}
    open={drawerOpen || !isMobile}
    loading={loadingProfiles}
    on:select={(event) => selectEmulator(event.detail)}
    on:close={() => (drawerOpen = false)}
  />

  <div class="main">
    <div class="top-bar">
      <div class="titles">
        <p class="eyebrow">CrossSave Cloud</p>
        <h1>Universal save syncing, tuned for every screen size.</h1>
      </div>
      <div class="actions">
        <button class="settings" on:click={openSettings} aria-label="Open settings">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path
              d="M10 3.5h4l.6 2.2a6.5 6.5 0 0 1 1.6.9l2.3-.6 2 3.4-1.7 1.6c.1.5.1 1 0 1.5l1.7 1.6-2 3.4-2.3-.6a6.5 6.5 0 0 1-1.6.9L14 20.5h-4l-.6-2.2a6.5 6.5 0 0 1-1.6-.9l-2.3.6-2-3.4 1.7-1.6a6.7 6.7 0 0 1 0-1.5L3.5 9.4l2-3.4 2.3.6a6.5 6.5 0 0 1 1.6-.9Z"
              fill="none"
              stroke="currentColor"
              stroke-width="1.6"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
            <circle cx="12" cy="12" r="2.5" fill="currentColor" />
          </svg>
          <span>Settings</span>
        </button>
        {#if isMobile}
          <button class="drawer-toggle" on:click={toggleDrawer} aria-label="Toggle emulator drawer">
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path
                d="M4 7.5h16a1 1 0 0 0 0-2H4a1 1 0 0 0 0 2Zm0 5h16a1 1 0 0 0 0-2H4a1 1 0 0 0 0 2Zm0 5h16a1 1 0 0 0 0-2H4a1 1 0 0 0 0 2Z"
                fill="currentColor"
              />
            </svg>
          </button>
        {/if}
      </div>
    </div>

    <div class="content-grid">
      <GameList games={selectedGames} emulatorName={selectedProfile?.name ?? ""} />
      <ProfilePreview profile={selectedProfile} loading={loadingProfiles} />
    </div>
  </div>
</div>

<style>
  .shell {
    display: flex;
    min-height: 100vh;
    width: 100%;
    background: var(--bg);
    color: var(--text);
  }

  .main {
    flex: 1;
    min-width: 0;
    padding: clamp(14px, 2vw, 32px);
    display: flex;
    flex-direction: column;
    gap: clamp(12px, 1.5vw, 24px);
  }

  .top-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: clamp(10px, 2vw, 16px);
    padding: clamp(10px, 1.5vw, 16px) clamp(12px, 1.8vw, 18px);
    border-radius: clamp(10px, 1vw, 14px);
    background: var(--surface);
    box-shadow: var(--shadow-strong);
    border: 1px solid var(--border);
  }

  .titles {
    display: flex;
    flex-direction: column;
    gap: clamp(4px, 1vw, 8px);
  }

  h1 {
    margin: 0;
    font-size: clamp(1.25rem, 1.2vw + 1rem, 1.75rem);
    font-weight: 700;
    color: var(--text);
  }

  .eyebrow {
    margin: 0;
    font-size: clamp(0.8rem, 0.5vw + 0.6rem, 0.95rem);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
    font-weight: 600;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: clamp(8px, 1vw, 12px);
  }

  .settings {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: clamp(8px, 1vw, 10px) clamp(10px, 1.4vw, 14px);
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--surface-muted);
    color: var(--text);
    cursor: pointer;
    box-shadow: var(--shadow);
    transition: transform 0.15s ease, box-shadow 0.2s ease, border-color 0.2s ease;
  }

  .settings:hover {
    transform: translateY(-1px);
    border-color: var(--accent);
    box-shadow: var(--shadow-strong);
  }

  .settings svg {
    width: clamp(18px, 4vw, 22px);
    height: clamp(18px, 4vw, 22px);
  }

  .settings span {
    font-weight: 700;
    font-size: clamp(0.9rem, 0.4vw + 0.8rem, 1rem);
  }

  .drawer-toggle {
    border: 1px solid var(--border);
    background: var(--surface);
    border-radius: 12px;
    padding: clamp(8px, 1vw, 10px);
    display: grid;
    place-items: center;
    cursor: pointer;
    color: var(--text);
    box-shadow: var(--shadow);
  }

  .drawer-toggle svg {
    width: clamp(18px, 4vw, 22px);
    height: clamp(18px, 4vw, 22px);
  }

  .content-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: clamp(12px, 1.8vw, 24px);
    align-items: start;
    min-height: 0;
  }

  @media (min-width: 700px) {
    .content-grid {
      grid-template-columns: minmax(320px, 1.6fr) minmax(260px, 1fr);
    }
  }
</style>
