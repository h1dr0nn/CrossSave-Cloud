<script lang="ts">
  import { onMount } from "svelte";
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
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.9), rgba(226, 232, 240, 0.75));
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
    background: rgba(255, 255, 255, 0.85);
    box-shadow: 0 10px 35px rgba(15, 23, 42, 0.08);
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
    color: #0f172a;
  }

  .eyebrow {
    margin: 0;
    font-size: clamp(0.8rem, 0.5vw + 0.6rem, 0.95rem);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #475569;
    font-weight: 600;
  }

  .drawer-toggle {
    border: 1px solid #cbd5e1;
    background: #fff;
    border-radius: 12px;
    padding: clamp(8px, 1vw, 10px);
    display: grid;
    place-items: center;
    cursor: pointer;
    color: #0f172a;
    box-shadow: 0 6px 18px rgba(15, 23, 42, 0.08);
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
