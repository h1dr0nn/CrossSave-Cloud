<script context="module" lang="ts">
  import type { GameEntry } from "../lib/uiTypes";
  import type { EmulatorProfile } from "../lib/api";

  const cachedProfiles: EmulatorProfile[] = [];
  let cachedGames = new Map<string, GameEntry[]>();
  let cachedSelectedEmulatorId: string | null = null;
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";

  import Sidebar from "./Sidebar.svelte";
  import GameList from "./GameList.svelte";
  import ProfilePreview from "./ProfilePreview.svelte";
  import AppHeader from "./AppHeader.svelte";
  import { listProfiles, scanSaveFiles } from "../lib/api";

  const DESKTOP_BREAKPOINT = 900;

  let profiles: EmulatorProfile[] = [];
  let selectedEmulatorId: string | null = null;
  let viewportWidth = 0; // Default to 0 to assume mobile first and avoid sidebar flash
  let drawerOpen = false;
  let loadingProfiles = true;

  onMount(async () => {
    if (cachedProfiles.length > 0) {
      profiles = [...cachedProfiles];
      selectedEmulatorId =
        cachedSelectedEmulatorId ?? cachedProfiles[0]?.emulator_id ?? null;
      if (selectedEmulatorId) {
        loadGames(selectedEmulatorId);
      }
      loadingProfiles = false;
      return;
    }

    try {
      const loadedProfiles = await listProfiles();
      profiles = loadedProfiles;
      cachedProfiles.push(...loadedProfiles);
      if (loadedProfiles.length > 0) {
        selectedEmulatorId = loadedProfiles[0].emulator_id;
        cachedSelectedEmulatorId = selectedEmulatorId;
        loadGames(selectedEmulatorId);
      }
    } catch (error) {
      console.error("Failed to load emulator profiles", error);
    } finally {
      loadingProfiles = false;
    }
  });

  $: selectedProfile =
    profiles.find((p) => p.emulator_id === selectedEmulatorId) ?? null;
  $: isDrawerMode = viewportWidth < DESKTOP_BREAKPOINT;
  $: sidebarVisible = drawerOpen || !isDrawerMode;
  $: selectedGames = selectedEmulatorId
    ? (cachedGames.get(selectedEmulatorId) ?? [])
    : [];

  $: if (!isDrawerMode && drawerOpen) {
    drawerOpen = false;
  }

  async function loadGames(emulatorId: string) {
    if (cachedGames.has(emulatorId)) return;

    try {
      const files = await scanSaveFiles(emulatorId);
      const games: GameEntry[] = files.map((file, index) => ({
        id: file.path, // Use path as ID
        emulatorId: emulatorId,
        name: file.name,
        icon: index % 3 === 0 ? "spark" : index % 2 === 0 ? "disc" : "console", // Keep random icon for now
        lastModified: new Date(file.modified).toISOString(),
      }));

      // Sort by modified desc
      games.sort(
        (a, b) =>
          new Date(b.lastModified).getTime() -
          new Date(a.lastModified).getTime()
      );

      cachedGames.set(emulatorId, games);
      // Trigger reactivity
      cachedGames = cachedGames;
    } catch (error) {
      console.error(`Failed to scan files for ${emulatorId}`, error);
    }
  }

  function selectEmulator(id: string) {
    selectedEmulatorId = id;
    cachedSelectedEmulatorId = id;
    loadGames(id);
    if (isDrawerMode) {
      drawerOpen = false;
    }
  }

  function toggleDrawer() {
    drawerOpen = !drawerOpen;
  }

  function openSettings() {
    goto("/settings", { keepFocus: true, noScroll: true });
  }
</script>

<svelte:window bind:innerWidth={viewportWidth} />

<div class="shell" data-mobile={isDrawerMode}>
  <Sidebar
    emulators={profiles}
    selectedId={selectedEmulatorId}
    isMobile={isDrawerMode}
    open={sidebarVisible}
    loading={loadingProfiles}
    on:select={(event) => selectEmulator(event.detail)}
    on:close={() => (drawerOpen = false)}
  />

  <div class="content">
    <div class="content-surface">
      <main class="content-body">
        <div class="header-wrapper">
          <AppHeader
            eyebrow="CrossSave Cloud"
            title="Dashboard"
            showMenu={isDrawerMode}
            onMenu={toggleDrawer}
            onBack={() => {}}
            sticky={false}
          >
            <button
              slot="actions"
              class="icon-button primary"
              on:click={openSettings}
              aria-label="Open settings"
            >
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path
                  d="M10.75 3.5h2.5l.58 2.25a5 5 0 0 1 1.4.82L17.5 6l1.75 2.76-1.72 1.26c.06.3.09.62.09.94 0 .32-.03.63-.09.93l1.72 1.27L17.5 16l-2.27-.57a5 5 0 0 1-1.4.82l-.58 2.25h-2.5l-.58-2.25a5 5 0 0 1-1.4-.82L6.5 16l-1.75-2.76 1.72-1.27A5 5 0 0 1 6.38 11c0-.32.03-.63.09-.93L4.75 8.76 6.5 6l2.27.57a5 5 0 0 1 1.4-.82Z"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
                <circle cx="12" cy="12" r="2.5" fill="currentColor" />
              </svg>
            </button>
          </AppHeader>
        </div>

        <div class="content-grid">
          <GameList
            games={selectedGames}
            emulatorName={selectedProfile?.name ?? ""}
          />
          <ProfilePreview profile={selectedProfile} loading={loadingProfiles} />
        </div>
      </main>
    </div>
  </div>
</div>

<style>
  .shell {
    display: grid;
    grid-template-columns: auto 1fr;
    min-height: 100vh;
    width: 100%;
    background: var(--bg);
    color: var(--text);
    align-items: stretch;
    overflow: hidden;
  }

  .content {
    min-width: 0;
    width: 100%;
    height: 100vh;
    display: grid;
    place-items: stretch;
    position: relative;
    overflow: hidden;
  }

  .content-surface {
    display: grid;
    grid-template-rows: 1fr;
    max-width: 1360px;
    margin: 0 auto;
    width: 100%;
    height: 100vh;
    overflow: hidden;
  }

  .content-body {
    min-height: 0;
    flex: 1;
    overflow: auto;
    scroll-behavior: smooth;
  }

  .header-wrapper {
    position: sticky;
    top: 0;
    z-index: 12;
    padding-top: max(clamp(16px, 3vw, 32px), env(safe-area-inset-top));
    padding-left: max(clamp(16px, 3vw, 32px), env(safe-area-inset-left));
    padding-right: max(clamp(16px, 3vw, 32px), env(safe-area-inset-right));
    padding-bottom: 14px;
    /* Ensure background covers content scrolling under */
    background: linear-gradient(to bottom, var(--bg) 80%, transparent);
    margin: 0 -1px; /* Fix sub-pixel gaps */
  }

  .content-body::-webkit-scrollbar {
    width: 6px;
  }

  .content-body::-webkit-scrollbar-thumb {
    background: color-mix(in srgb, var(--accent) 35%, transparent);
    border-radius: 12px;
  }

  .content-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 16px;
    align-items: start;
    min-height: 0;
    padding-left: max(clamp(16px, 3vw, 32px), env(safe-area-inset-left));
    padding-right: max(clamp(16px, 3vw, 32px), env(safe-area-inset-right));
    padding-bottom: max(clamp(16px, 3vw, 32px), env(safe-area-inset-bottom));
  }

  @media (max-width: 899px) {
    .shell {
      grid-template-columns: 1fr;
    }

    .content {
      height: auto;
      min-height: 100vh;
    }

    .content-surface {
      height: auto;
    }
  }
</style>
