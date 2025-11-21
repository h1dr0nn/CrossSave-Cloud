<script lang="ts">
import { onMount } from "svelte";
import { goto } from "$app/navigation";

import Sidebar from "./Sidebar.svelte";
import GameList from "./GameList.svelte";
import ProfilePreview from "./ProfilePreview.svelte";
import type { GameEntry } from "../lib/uiTypes";
import { listProfiles, type EmulatorProfile } from "../lib/api";

const cachedProfiles: EmulatorProfile[] = [];
const cachedGames = new Map<string, GameEntry[]>();
let cachedSelectedEmulatorId: string | null = null;

let profiles: EmulatorProfile[] = [];
let selectedEmulatorId: string | null = null;
let viewportWidth = 960;
let drawerOpen = false;
let loadingProfiles = true;

onMount(async () => {
  if (cachedProfiles.length > 0) {
    profiles = [...cachedProfiles];
    selectedEmulatorId = cachedSelectedEmulatorId ?? cachedProfiles[0]?.emulator_id ?? null;
    preloadGames(cachedProfiles);
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
      preloadGames(loadedProfiles);
    }
  } catch (error) {
    console.error("Failed to load emulator profiles", error);
  } finally {
    loadingProfiles = false;
  }
});

  $: selectedProfile = profiles.find((p) => p.emulator_id === selectedEmulatorId) ?? null;
  $: isMobile = viewportWidth < 720;
  $: selectedGames = selectedEmulatorId ? cachedGames.get(selectedEmulatorId) ?? [] : [];

  function preloadGames(items: EmulatorProfile[]) {
    items.forEach((profile) => {
      if (!cachedGames.has(profile.emulator_id)) {
        cachedGames.set(profile.emulator_id, generateGames(profile));
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
    cachedSelectedEmulatorId = id;
    if (isMobile) {
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

  <div class="content">
    <div class="content-inner">
      <header class="app-header">
        <div class="leading">
          {#if isMobile}
            <button class="icon-button" on:click={toggleDrawer} aria-label="Toggle emulator drawer">
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d="M4 7h16M4 12h16M4 17h16" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
              </svg>
            </button>
          {/if}
          <div class="titles">
            <p class="eyebrow">CrossSave Cloud</p>
            <h1>Dashboard</h1>
          </div>
        </div>
        <button class="icon-button primary" on:click={openSettings} aria-label="Open settings">
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
      </header>

      <div class="content-grid">
        <GameList games={selectedGames} emulatorName={selectedProfile?.name ?? ""} />
        <ProfilePreview profile={selectedProfile} loading={loadingProfiles} />
      </div>
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
  }

  .content {
    min-width: 0;
    width: 100%;
    display: flex;
    justify-content: center;
  }

  .content-inner {
    flex: 1;
    max-width: 1320px;
    padding: 24px clamp(16px, 3vw, 32px) 28px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .app-header {
    position: sticky;
    top: 0;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 14px 18px;
    border-radius: var(--radius);
    background: color-mix(in srgb, var(--surface) 92%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 90%, transparent);
    box-shadow: var(--shadow-soft);
    backdrop-filter: blur(14px);
  }

  .leading {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .titles {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  h1 {
    margin: 0;
    font-size: clamp(1.2rem, 0.5vw + 1.05rem, 1.6rem);
    font-weight: 700;
    color: var(--text);
    letter-spacing: -0.02em;
  }

  .eyebrow {
    margin: 0;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--muted);
    font-weight: 600;
  }

  .icon-button {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    border: 1px solid var(--border);
    background: var(--surface);
    display: grid;
    place-items: center;
    color: var(--text);
    box-shadow: var(--shadow-soft);
    cursor: pointer;
    transition: transform 0.15s ease, box-shadow 0.2s ease, border-color 0.2s ease;
  }

  .icon-button.primary {
    background: var(--surface-muted);
  }

  .icon-button:hover {
    transform: translateY(-1px);
    border-color: var(--accent);
    box-shadow: var(--shadow);
  }

  .icon-button svg {
    width: 22px;
    height: 22px;
  }

  .content-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 16px;
    align-items: start;
    min-height: 0;
  }

  @media (min-width: 860px) {
    .content-grid {
      grid-template-columns: minmax(360px, 1.2fr) minmax(320px, 1fr);
    }
  }

  @media (max-width: 760px) {
    .shell {
      grid-template-columns: 1fr;
    }

    .content-inner {
      padding: 16px 14px 20px;
      gap: 12px;
    }

    .app-header {
      position: static;
    }
  }
</style>
