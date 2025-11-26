<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";

  import Sidebar from "./Sidebar.svelte";
  import AppHeader from "./AppHeader.svelte";
  import ScanPopup from "../shared/ScanPopup.svelte";
  import {
    listProfiles,
    scanSaveFiles,
    type EmulatorProfile,
  } from "../../lib/api";
  import { extractGameName, getIconVariant } from "../../lib/utils";
  import {
    profilesStore,
    gamesCacheStore,
    selectedEmulatorIdStore,
  } from "../../lib/stores";
  import type { GameEntry } from "../../lib/uiTypes";
  import GameList from "../game/GameList.svelte";
  import EmulatorList from "../emulator/EmulatorList.svelte";
  import ProfilePreview from "../emulator/ProfilePreview.svelte";
  import Notifications from "../shared/Notifications.svelte";

  const DESKTOP_BREAKPOINT = 900;

  let viewportWidth = 0; // Default to 0 to assume mobile first and avoid sidebar flash
  let drawerOpen = false;
  let loadingProfiles = true;

  // Scan state
  let showScanPopup = false;
  let isScanning = false;
  let scanProgress = "";
  let loadingGames = false;

  onMount(async () => {
    const startTime = Date.now();

    // Check if we already have profiles in store
    if ($profilesStore.length > 0) {
      loadingProfiles = false;
      if (!$selectedEmulatorIdStore) {
        selectedEmulatorIdStore.set($profilesStore[0].emulator_id);
      }
      // If we have a selected emulator, ensure its games are loaded (or at least checked)
      if ($selectedEmulatorIdStore) {
        loadGames($selectedEmulatorIdStore);
      }
      return;
    }

    try {
      // Add a timeout to prevent infinite loading
      const timeoutPromise = new Promise<EmulatorProfile[]>((_, reject) =>
        setTimeout(() => reject(new Error("Profile load timeout")), 5000)
      );

      const loadedProfiles = await Promise.race([
        listProfiles(),
        timeoutPromise,
      ]);

      profilesStore.set(loadedProfiles);

      if (loadedProfiles.length > 0) {
        if (!$selectedEmulatorIdStore) {
          selectedEmulatorIdStore.set(loadedProfiles[0].emulator_id);
        }

        // Check if this is the very first time the app is launched
        const hasLaunchedBefore = localStorage.getItem("hasLaunchedBefore");

        if (!hasLaunchedBefore) {
          // First launch ever - show popup
          showScanPopup = true;
          localStorage.setItem("hasLaunchedBefore", "true");
        } else {
          // Subsequent launches - no auto-scan
          // scanAll();
        }
      }
    } catch (error) {
      console.error("Failed to load emulator profiles", error);
      // If timeout or error, ensure we stop loading
    } finally {
      // Ensure minimum 1s loading time
      const elapsed = Date.now() - startTime;
      const remaining = Math.max(0, 1000 - elapsed);

      if (remaining > 0) {
        await new Promise((resolve) => setTimeout(resolve, remaining));
      }

      loadingProfiles = false;
    }
  });

  $: selectedProfile =
    $profilesStore.find((p) => p.emulator_id === $selectedEmulatorIdStore) ??
    null;
  $: isDrawerMode = viewportWidth < DESKTOP_BREAKPOINT;
  $: sidebarVisible = drawerOpen || !isDrawerMode;
  $: selectedGames = $selectedEmulatorIdStore
    ? ($gamesCacheStore.get($selectedEmulatorIdStore) ?? [])
    : [];

  $: if (!isDrawerMode && drawerOpen) {
    drawerOpen = false;
  }

  async function loadGames(emulatorId: string, force = false) {
    if (!force && $gamesCacheStore.has(emulatorId)) return;

    const startTime = Date.now();
    loadingGames = true;
    try {
      const files = await scanSaveFiles(emulatorId);
      const games: GameEntry[] = files.map((file) => {
        const gameName = extractGameName(file.name);
        // DEBUG: Show first game name
        if (files.indexOf(file) === 0) {
          console.log(
            `[DEBUG] First game extracted: "${gameName}" from "${file.name}"`
          );
        }
        return {
          id: gameName, // Use extracted name as ID (consistent with UI)
          emulatorId: emulatorId,
          name: gameName,
          icon: getIconVariant(file.path),
          lastModified: new Date(file.modified).toISOString(),
        };
      });

      // Sort by modified desc
      games.sort(
        (a, b) =>
          new Date(b.lastModified).getTime() -
          new Date(a.lastModified).getTime()
      );

      gamesCacheStore.update((cache) => {
        const newCache = new Map(cache);
        newCache.set(emulatorId, games);
        return newCache;
      });
    } catch (error) {
      console.error(`Failed to scan files for ${emulatorId}`, error);
    } finally {
      // Ensure minimum 1s loading time
      const elapsed = Date.now() - startTime;
      const remaining = Math.max(0, 1000 - elapsed);

      if (remaining > 0) {
        await new Promise((resolve) => setTimeout(resolve, remaining));
      }

      loadingGames = false;
    }
  }

  async function scanAll() {
    isScanning = true;
    try {
      const profiles = $profilesStore;
      for (let i = 0; i < profiles.length; i++) {
        const profile = profiles[i];
        scanProgress = `Scanning ${profile.name} (${i + 1}/${profiles.length})...`;
        await loadGames(profile.emulator_id, true);
      }
    } catch (e) {
      console.error("Scan all failed", e);
    } finally {
      isScanning = false;
      showScanPopup = false;
    }
  }

  function selectEmulator(id: string) {
    selectedEmulatorIdStore.set(id);
    // Always force reload on tab switch as requested
    loadGames(id, true);
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

{#if showScanPopup}
  <ScanPopup
    scanning={isScanning}
    progress={scanProgress}
    on:scan={scanAll}
    on:cancel={() => (showScanPopup = false)}
  />
{/if}

<Notifications />

<div
  class="shell"
  data-mobile={isDrawerMode}
  class:locked={isDrawerMode && drawerOpen}
>
  <Sidebar
    emulators={$profilesStore}
    selectedId={$selectedEmulatorIdStore}
    isMobile={isDrawerMode}
    isOpen={sidebarVisible}
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
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path
                  d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.09a2 2 0 0 1-1-1.74v-.47a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
                />
                <circle cx="12" cy="12" r="3" />
              </svg>
            </button>
          </AppHeader>
        </div>

        <div class="content-grid">
          <GameList
            games={selectedGames}
            emulatorName={selectedProfile?.name ?? ""}
            loading={loadingGames}
            on:reload={() =>
              $selectedEmulatorIdStore &&
              loadGames($selectedEmulatorIdStore, true)}
          />
          <ProfilePreview profile={selectedProfile} />
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

  .shell.locked .content-body {
    overflow: hidden;
    pointer-events: none;
    user-select: none;
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
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    scroll-behavior: smooth;
    padding: clamp(16px, 3vw, 32px);
    padding-top: max(clamp(16px, 3vw, 32px), env(safe-area-inset-top));
    padding-bottom: max(clamp(16px, 3vw, 32px), env(safe-area-inset-bottom));
    padding-left: max(clamp(16px, 3vw, 32px), env(safe-area-inset-left));
    padding-right: max(clamp(16px, 3vw, 32px), env(safe-area-inset-right));
  }

  .header-wrapper {
    margin-bottom: clamp(16px, 3vw, 32px);
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
    gap: clamp(16px, 3vw, 32px);
    align-items: start;
    min-height: 0;
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
