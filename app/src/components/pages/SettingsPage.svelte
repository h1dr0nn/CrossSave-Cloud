<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import AppHeader from "../layout/AppHeader.svelte";
  import { userEmail, isLoggedIn, cloudStore } from "$lib/stores/cloudStore";

  onMount(async () => {
    // Initialize cloud store to restore login state
    await cloudStore.initialize();
  });

  function goBack() {
    goto("/", { keepFocus: true, noScroll: true });
  }

  function openCloud() {
    goto("/cloud", { keepFocus: true, noScroll: true });
  }
</script>

<section class="settings-page">
  <div class="content-surface">
    <main class="content-body">
      <div class="header-wrapper">
        <AppHeader
          title="Settings"
          showBack
          onBack={goBack}
          onMenu={() => {}}
          sticky={false}
        />
      </div>

      <div class="settings-container">
        <!-- Account Section -->
        {#if $isLoggedIn}
          <div class="section-group">
            <div class="section-header">
              <p class="section-title">Account</p>
            </div>
            <div class="settings-card">
              <button class="setting-row clickable" on:click={openCloud}>
                <div class="setting-info">
                  <div class="setting-icon cloud">
                    <svg viewBox="0 0 24 24" fill="none">
                      <path
                        d="M18 10h-1.26A8 8 0 1 0 9 17h9a5 5 0 0 0 0-10z"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      />
                    </svg>
                  </div>
                  <div class="setting-content">
                    <p class="setting-label">Cloud Sync</p>
                    <p class="setting-value">{$userEmail || "Not signed in"}</p>
                  </div>
                </div>
                <svg class="chevron" viewBox="0 0 24 24" fill="none">
                  <path
                    d="M9 18l6-6-6-6"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
              </button>
            </div>
          </div>
        {:else}
          <div class="section-group">
            <div class="section-header">
              <p class="section-title">Account</p>
            </div>
            <div class="settings-card">
              <button class="setting-row clickable" on:click={openCloud}>
                <div class="setting-info">
                  <div class="setting-icon cloud">
                    <svg viewBox="0 0 24 24" fill="none">
                      <path
                        d="M18 10h-1.26A8 8 0 1 0 9 17h9a5 5 0 0 0 0-10z"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      />
                    </svg>
                  </div>
                  <div class="setting-content">
                    <p class="setting-label">Sign in to Cloud</p>
                    <p class="setting-value">Sync your saves across devices</p>
                  </div>
                </div>
                <svg class="chevron" viewBox="0 0 24 24" fill="none">
                  <path
                    d="M9 18l6-6-6-6"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
              </button>
            </div>
          </div>
        {/if}

        <!-- General Settings -->
        <div class="section-group">
          <div class="section-header">
            <p class="section-title">General</p>
          </div>
          <div class="settings-card">
            <a href="/settings/profiles" class="setting-row clickable">
              <div class="setting-info">
                <div class="setting-icon profiles">
                  <svg viewBox="0 0 24 24" fill="none">
                    <rect
                      x="6"
                      y="4"
                      width="12"
                      height="16"
                      rx="2"
                      stroke="currentColor"
                      stroke-width="2"
                    />
                  </svg>
                </div>
                <div class="setting-content">
                  <p class="setting-label">Emulator Profiles</p>
                </div>
              </div>
              <svg class="chevron" viewBox="0 0 24 24" fill="none">
                <path
                  d="M9 18l6-6-6-6"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </a>

            <div class="divider"></div>

            <a href="/settings/cloud" class="setting-row clickable">
              <div class="setting-info">
                <div class="setting-icon cloud-mode">
                  <svg viewBox="0 0 24 24" fill="none">
                    <path
                      d="M18 10h-1.26A8 8 0 1 0 9 17h9a5 5 0 0 0 0-10z"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    />
                  </svg>
                </div>
                <div class="setting-content">
                  <p class="setting-label">Cloud Mode</p>
                </div>
              </div>
              <svg class="chevron" viewBox="0 0 24 24" fill="none">
                <path
                  d="M9 18l6-6-6-6"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </a>

            <div class="divider"></div>

            <a href="/settings/backup" class="setting-row clickable">
              <div class="setting-info">
                <div class="setting-icon backup">
                  <svg viewBox="0 0 24 24" fill="none">
                    <path
                      d="M12 2v20M12 2l-4 4M12 2l4 4"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    />
                  </svg>
                </div>
                <div class="setting-content">
                  <p class="setting-label">Backup</p>
                </div>
              </div>
              <svg class="chevron" viewBox="0 0 24 24" fill="none">
                <path
                  d="M9 18l6-6-6-6"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </a>

            <div class="divider"></div>

            <a href="/settings/storage" class="setting-row clickable">
              <div class="setting-info">
                <div class="setting-icon storage">
                  <svg viewBox="0 0 24 24" fill="none">
                    <path
                      d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    />
                  </svg>
                </div>
                <div class="setting-content">
                  <p class="setting-label">Storage</p>
                </div>
              </div>
              <svg class="chevron" viewBox="0 0 24 24" fill="none">
                <path
                  d="M9 18l6-6-6-6"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </a>
          </div>
        </div>

        <!-- Appearance -->
        <div class="section-group">
          <div class="section-header">
            <p class="section-title">Appearance</p>
          </div>
          <div class="settings-card">
            <a href="/settings/theme" class="setting-row clickable">
              <div class="setting-info">
                <div class="setting-icon theme">
                  <svg viewBox="0 0 24 24" fill="none">
                    <circle
                      cx="12"
                      cy="12"
                      r="5"
                      stroke="currentColor"
                      stroke-width="2"
                    />
                    <path
                      d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                    />
                  </svg>
                </div>
                <div class="setting-content">
                  <p class="setting-label">Theme</p>
                </div>
              </div>
              <svg class="chevron" viewBox="0 0 24 24" fill="none">
                <path
                  d="M9 18l6-6-6-6"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </a>
          </div>
        </div>
      </div>
    </main>
  </div>
</section>

<style>
  .settings-page {
    display: grid;
    grid-template-columns: 1fr;
    min-height: 100vh;
    width: 100%;
    background: var(--bg);
    color: var(--text);
  }

  .content-surface {
    display: grid;
    grid-template-rows: 1fr;
    max-width: 1360px;
    margin: 0 auto;
    width: 100%;
    min-height: 100vh;
  }

  .content-body {
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

  .settings-container {
    width: 100%;
    display: grid;
    gap: 32px;
    align-content: start;
  }

  .section-group {
    display: grid;
    gap: 8px;
  }

  .section-header {
    padding: 0 16px;
  }

  .section-title {
    margin: 0;
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
  }

  .settings-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
    box-shadow: var(--shadow-soft);
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    gap: 12px;
    text-decoration: none;
    color: inherit;
    width: 100%;
    border: none;
    background: none;
    text-align: left;
    font-family: inherit;
    font-size: inherit;
  }

  .setting-row.clickable {
    cursor: pointer;
    transition: background 0.2s;
  }

  .setting-row.clickable:hover {
    background: color-mix(in srgb, var(--surface-muted) 30%, transparent);
  }

  .setting-info {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    min-width: 0;
  }

  .setting-icon {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    flex-shrink: 0;
  }

  .setting-icon svg {
    width: 18px;
    height: 18px;
    color: white;
  }

  .setting-icon.cloud {
    background: linear-gradient(135deg, #3b82f6, #2563eb);
  }

  .setting-icon.cloud-mode {
    background: linear-gradient(135deg, #06b6d4, #0891b2);
  }

  .setting-icon.profiles {
    background: linear-gradient(135deg, #8b5cf6, #7c3aed);
  }

  .setting-icon.backup {
    background: linear-gradient(135deg, #10b981, #059669);
  }

  .setting-icon.storage {
    background: linear-gradient(135deg, #f59e0b, #d97706);
  }

  .setting-icon.theme {
    background: linear-gradient(135deg, #ec4899, #db2777);
  }

  .setting-content {
    flex: 1;
    min-width: 0;
  }

  .setting-label {
    margin: 0;
    font-weight: 500;
    color: var(--text);
  }

  .setting-value {
    margin: 4px 0 0 0;
    font-size: 0.9rem;
    color: var(--muted);
  }

  .chevron {
    width: 20px;
    height: 20px;
    color: var(--muted);
    flex-shrink: 0;
  }

  .divider {
    height: 1px;
    background: var(--border);
    margin: 0 16px;
    opacity: 0.6;
  }

  @media (max-width: 640px) {
    .settings-container {
      gap: 24px;
    }
  }
</style>
