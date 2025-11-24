<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { goto } from "$app/navigation";
  import AppHeader from "../layout/AppHeader.svelte";
  import {
    cloudStore,
    isLoggedIn,
    userEmail,
    isSyncing,
  } from "$lib/stores/cloudStore";
  import type {
    CloudDevice,
    SyncStatus,
    CloudVersion,
  } from "$lib/stores/cloudStore";
  import { pushError, pushInfo, pushSuccess } from "$lib/notifications";

  // Login state
  let email = "";
  let password = "";
  let loginError = "";

  function goBack() {
    goto("/settings", { keepFocus: true, noScroll: true });
  }

  // Device management
  let deviceId = "";
  let devices: CloudDevice[] = [];

  // Sync status
  const syncStatusStore = cloudStore.syncStatus;
  const cloudVersionsStore = cloudStore.cloudVersions;
  const downloadStateStore = cloudStore.downloadState;
  const onlineStatusStore = cloudStore.onlineStatus;
  const devicesStore = cloudStore.devices;

  let syncStatus: SyncStatus | null = null;
  let selectedVersions: CloudVersion[] = [];
  let syncMessage = "";
  let downloadState: {
    versionId: string | null;
    progress: number;
    status: "idle" | "downloading" | "completed" | "error";
    path: string | null;
    error: string | null;
  } = {
    versionId: null,
    progress: 0,
    status: "idle",
    path: null,
    error: null,
  };
  let onlineStatus = "online";
  let hasLoadedAfterLogin = false;
  let lastDownloadError = "";
  let lastDownloadPath = "";

  // Cloud versions
  let selectedGame = "";
  let loadingVersions = false;

  $: syncStatus = $syncStatusStore;
  $: selectedVersions = $cloudVersionsStore.get(selectedGame) ?? [];
  $: downloadState = $downloadStateStore;
  $: onlineStatus = $onlineStatusStore;
  $: devices = $devicesStore;

  $: if ($isLoggedIn && !hasLoadedAfterLogin) {
    hasLoadedAfterLogin = true;
    initializeAfterLogin();
  } else if (!$isLoggedIn) {
    hasLoadedAfterLogin = false;
  }

  $: if (
    downloadState.status === "completed" &&
    downloadState.path &&
    downloadState.path !== lastDownloadPath
  ) {
    pushSuccess(`Download complete (${downloadState.versionId ?? ""})`);
    lastDownloadPath = downloadState.path;
    if (selectedGame) {
      loadCloudVersions();
    }
  }

  $: if (
    downloadState.status === "error" &&
    downloadState.error &&
    downloadState.error !== lastDownloadError
  ) {
    pushError(`Download failed: ${downloadState.error}`);
    lastDownloadError = downloadState.error;
  }

  $: {
    let nextMessage: string | null = null;

    if (downloadState.status === "downloading") {
      nextMessage = `Downloading ${Math.round(downloadState.progress)}%`;
    } else if (downloadState.status === "error" && downloadState.error) {
      nextMessage = `Download error: ${downloadState.error}`;
    } else if ($isSyncing && syncStatus) {
      nextMessage = `Syncing (${syncStatus.queue_length} queued)`;
    } else if (syncStatus && syncStatus.queue_length > 0) {
      nextMessage = `${syncStatus.queue_length} pending`;
    } else if (downloadState.status === "completed") {
      nextMessage = "Sync complete";
    }

    if (nextMessage !== null) {
      syncMessage = nextMessage;
    }
  }

  onMount(async () => {
    await cloudStore.initialize();
    if (get(isLoggedIn)) {
      await initializeAfterLogin();
    }
  });

  async function initializeAfterLogin() {
    await loadCloudStatus();
    await loadSyncStatus();
    await refreshDevices();
  }

  async function handleLogin() {
    loginError = "";
    const result = await cloudStore.login(email, password);
    if (result.success) {
      await initializeAfterLogin();
    } else {
      loginError = result.error ?? "Invalid credentials";
    }
  }

  async function handleLogout() {
    await cloudStore.logout();
    deviceId = "";
    devices = [];
  }

  async function loadCloudStatus() {
    try {
      const status = await cloudStore.getCloudStatus();
      deviceId = status.device_id;
      onlineStatus = status.connected ? "online" : "offline";
    } catch (error) {
      loginError =
        typeof error === "string"
          ? error
          : "Failed to load cloud status";
      pushError(loginError);
    }
  }

  async function loadSyncStatus() {
    try {
      await cloudStore.getSyncStatus();
    } catch (error) {
      console.error("Failed to load sync status:", error);
    }
  }

  async function handleSyncNow() {
    try {
      syncMessage = "Sync requested...";
      await cloudStore.forceSyncNow();
      await loadSyncStatus();
    } catch (error) {
      syncMessage = "Sync failed: " + error;
    }
  }

  async function handleClearQueue() {
    if (confirm("Clear all pending uploads?")) {
      try {
        await cloudStore.clearSyncQueue();
        await loadSyncStatus();
        syncMessage = "Queue cleared";
        setTimeout(() => (syncMessage = ""), 3000);
      } catch (error) {
        console.error("Failed to clear queue:", error);
      }
    }
  }

  async function refreshDevices() {
    try {
      await cloudStore.listDevices();
    } catch (error) {
      pushError("Failed to load devices");
    }
  }

  async function removeDevice(device: CloudDevice) {
    if (!confirm(`Remove device ${device.name}?`)) return;
    try {
      await cloudStore.removeDevice(device.device_id);
      pushSuccess("Device removed");
    } catch (error) {
      pushError(`Failed to remove device: ${error}`);
    }
  }

  async function loadCloudVersions() {
    if (!selectedGame) return;

    loadingVersions = true;
    try {
      await cloudStore.listCloudVersions(selectedGame, 10);
    } catch (error) {
      console.error("Failed to load cloud versions:", error);
      pushError("Failed to load cloud versions");
    } finally {
      loadingVersions = false;
    }
  }

  async function handleDownload(versionId: string) {
    try {
      await cloudStore.downloadCloudVersion(selectedGame, versionId);
      pushInfo("Download started");
    } catch (error) {
      pushError("Download failed: " + error);
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }
</script>

<section class="settings-page">
  <div class="content-surface">
    <main class="content-body">
      <div class="header-wrapper">
        <AppHeader
          title="Cloud Sync"
          showBack
          onBack={goBack}
          onMenu={() => {}}
          sticky={false}
        />
      </div>

      <div class="settings-container">
        {#if !$isLoggedIn}
          <!-- Login Form -->
          <div class="section-group">
            <div class="section-header">
              <p class="section-title">Sign In</p>
            </div>
            <div class="settings-card">
              <div class="card-content">
                <p class="description">
                  Connect to CrossSave Cloud to sync your saves
                </p>

                <form on:submit|preventDefault={handleLogin}>
                  <div class="input-stack">
                    <div class="form-group">
                      <label for="email">Email</label>
                      <input
                        id="email"
                        type="email"
                        bind:value={email}
                        placeholder="your@email.com"
                        required
                      />
                    </div>

                    <div class="form-group">
                      <label for="password">Password</label>
                      <input
                        id="password"
                        type="password"
                        bind:value={password}
                        placeholder="••••••••"
                        required
                      />
                    </div>
                  </div>

                {#if loginError}
                  <p class="error">{loginError}</p>
                {/if}

                  <div class="form-actions">
                    <button type="submit" class="btn-primary btn-full">
                      Sign In
                    </button>
                  </div>
                </form>
              </div>
            </div>
          </div>
        {:else}
          <!-- Account Info -->
          <div class="section-group">
            <div class="section-header">
              <p class="section-title">Account</p>
            </div>
            <div class="settings-card">
              <div class="setting-row">
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
                    <p class="setting-label">Logged in as</p>
                    <p class="setting-value">{$userEmail || "Cloud account"}</p>
                    <p class="setting-meta">Device ID: {deviceId || "Unknown"}</p>
                  </div>
                </div>
                <span
                  class:offline={onlineStatus === "offline"}
                  class="status-pill"
                  aria-live="polite"
                >
                  {onlineStatus === "online" ? "Online" : "Offline"}
                </span>
                <button class="btn-secondary btn-small" on:click={handleLogout}>
                  Logout
                </button>
              </div>
            </div>
          </div>

          <!-- Sync Status -->
          <div class="section-group">
            <div class="section-header">
              <p class="section-title">Sync Status</p>
            </div>
            <div class="settings-card">
              <div class="card-content">
                {#if syncStatus}
                  <div class="status-grid">
                    <div class="status-item">
                      <span class="label">Queue</span>
                      <span class="value">{syncStatus.queue_length}</span>
                    </div>
                    <div class="status-item">
                      <span class="label">Connection</span>
                      <span
                        class={`value ${onlineStatus === "online" ? "syncing" : "idle"}`}
                      >
                        {onlineStatus === "online" ? "Online" : "Offline"}
                      </span>
                    </div>
                    <div class="status-item">
                      <span class="label">Status</span>
                      <span class="value {$isSyncing ? 'syncing' : 'idle'}">
                        {$isSyncing ? "Syncing..." : "Idle"}
                      </span>
                    </div>
                  </div>
                  {#if syncStatus.last_sync}
                    <p class="last-sync">
                      Last synced: {new Date(
                        syncStatus.last_sync,
                      ).toLocaleString()}
                    </p>
                  {/if}
                {/if}

                {#if syncStatus?.active_job}
                  <div class="active-job">
                    <div>
                      <p class="label">Uploading</p>
                      <p class="mono">{syncStatus.active_job.version_id}</p>
                    </div>
                    <p class="hint">Game: {syncStatus.active_job.game_id}</p>
                  </div>
                {/if}

                {#if downloadState.status !== "idle" && downloadState.versionId}
                  <div class="progress-wrapper">
                    <div class="progress-header">
                      <span class="label">Download</span>
                      <span class="mono">{downloadState.versionId}</span>
                    </div>
                    <div class="progress-bar">
                      <span style={`width: ${downloadState.progress}%`}></span>
                    </div>
                    {#if downloadState.error}
                      <p class="error-text">{downloadState.error}</p>
                    {/if}
                  </div>
                {/if}

                {#if syncMessage}
                  <p class="sync-message">{syncMessage}</p>
                {/if}

                <div class="actions-row">
                  <button
                    class="btn-primary"
                    on:click={handleSyncNow}
                    disabled={$isSyncing}
                  >
                    {$isSyncing ? "Syncing..." : "Sync Now"}
                  </button>

                  <button
                    class="btn-secondary"
                    on:click={handleClearQueue}
                    disabled={!syncStatus || syncStatus.queue_length === 0}
                  >
                    Clear Queue
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Device Management -->
          <div class="section-group">
            <div class="section-header">
              <p class="section-title">Devices</p>
            </div>
            <div class="settings-card">
              <div class="card-content">
                <div class="device-info">
                  <span class="label">Current Device ID</span>
                  <code class="device-id">{deviceId}</code>
                </div>

                <div class="divider"></div>

                <p class="subsection-title">Registered Devices</p>
                <div class="devices-list">
                  {#if devices.length === 0}
                    <p class="empty">No registered devices yet</p>
                  {:else}
                    {#each devices as device}
                      <div class="device-item">
                        <div class="device-details">
                          <strong>{device.name}</strong>
                          <span class="device-meta"
                            >Last sync: {new Date(
                              (device.last_sync || 0) * 1000,
                            ).toLocaleString()}</span
                          >
                          <span class="device-meta mono"
                            >ID: {device.device_id.substring(0, 10)}...</span
                          >
                        </div>
                        <div class="device-actions">
                          <button
                            class="btn-danger btn-small"
                            on:click={() => removeDevice(device)}
                          >
                            Remove
                          </button>
                        </div>
                      </div>
                    {/each}
                  {/if}
                </div>
              </div>
            </div>
          </div>

          <!-- Cloud Version History -->
          <div class="section-group">
            <div class="section-header">
              <p class="section-title">Version History</p>
            </div>
            <div class="settings-card">
              <div class="card-content">
                <div class="form-group search-group">
                  <label for="game-select">Search Game ID</label>
                  <div class="search-input">
                    <input
                      id="game-select"
                      type="text"
                      bind:value={selectedGame}
                      placeholder="Enter game ID..."
                      on:blur={loadCloudVersions}
                    />
                    <button
                      class="btn-secondary"
                      on:click={loadCloudVersions}
                      disabled={!selectedGame || loadingVersions}
                    >
                      Search
                    </button>
                  </div>
                </div>

                {#if loadingVersions}
                  <p class="loading">Loading versions...</p>
                {:else if selectedVersions.length > 0}
                  <div class="versions-list">
                    {#each selectedVersions as version}
                      <div class="version-item">
                        <div class="version-info">
                          <div class="version-header">
                            <strong
                              >{version.version_id.substring(0, 8)}...</strong
                            >
                            <span class="version-date"
                              >{formatDate(version.timestamp)}</span
                            >
                          </div>
                          <div class="version-meta">
                            <span
                              >{formatBytes(
                                version.total_size_bytes ?? version.size_bytes,
                              )}</span
                            >
                            <span class="separator">•</span>
                            <span class="mono"
                              >Hash: {version.hash.substring(0, 10)}...</span
                            >
                          </div>
                          <div class="version-meta">
                            <span
                              >Device: {version.device_id.substring(0, 8)}...</span
                            >
                            <span class="separator">•</span>
                            <span
                              >Files: {version.file_list?.length ?? 0}</span
                            >
                          </div>
                          {#if downloadState.versionId === version.version_id}
                            <div class="progress-bar small">
                              <span
                                style={`width: ${downloadState.progress}%`}
                              ></span>
                            </div>
                            {#if downloadState.error}
                              <p class="error-text">{downloadState.error}</p>
                            {/if}
                          {/if}
                        </div>
                        <button
                          class="btn-secondary btn-small"
                          on:click={() => handleDownload(version.version_id)}
                        >
                          Download
                        </button>
                      </div>
                    {/each}
                  </div>
                {:else if selectedGame}
                  <p class="empty">No cloud versions found for this game</p>
                {/if}
              </div>
            </div>
          </div>
        {/if}
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

  .card-content {
    padding: 24px;
  }

  .description {
    color: var(--text-secondary);
    margin-bottom: 24px;
    font-size: 0.95rem;
  }

  /* Form Styles */
  .input-stack {
    display: grid;
    gap: 12px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: color-mix(in srgb, var(--surface-muted) 70%, transparent);
    box-shadow: inset 0 1px 0
      color-mix(in srgb, var(--surface) 40%, transparent);
    margin-bottom: 24px;
  }

  .form-group {
    display: grid;
    gap: 8px;
  }

  .input-stack .form-group {
    padding: 0;
  }

  label {
    display: block;
    font-weight: 500;
    color: var(--text-primary);
    font-size: 0.9rem;
  }

  input[type="email"],
  input[type="password"],
  input[type="text"] {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 0.95rem;
    transition: all 0.2s;
  }

  input:focus {
    outline: none;
    border-color: var(--accent-color);
    box-shadow: 0 0 0 2px
      color-mix(in srgb, var(--accent-color) 20%, transparent);
    background: var(--surface);
  }

  .form-actions {
    margin-top: 12px;
  }

  /* Buttons */
  .btn-primary,
  .btn-secondary,
  .btn-danger {
    padding: 0.6rem 1.2rem;
    border: none;
    border-radius: 8px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.95rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .btn-full {
    width: 100%;
  }

  .btn-small {
    padding: 0.4rem 0.8rem;
    font-size: 0.85rem;
  }

  .btn-primary {
    background: var(--accent);
    color: #fff;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent-strong);
  }

  .btn-secondary {
    background: var(--surface-muted);
    color: var(--text-primary);
    border: 1px solid var(--border);
  }

  .btn-secondary:hover {
    background: var(--bg-hover);
  }

  .btn-danger {
    background: color-mix(in srgb, var(--danger) 10%, transparent);
    color: var(--danger);
    border: 1px solid color-mix(in srgb, var(--danger) 20%, transparent);
  }

  .btn-danger:hover {
    background: var(--danger);
    color: white;
  }

  .btn-primary:disabled,
  .btn-secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Account Row */
  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px;
    gap: 12px;
  }

  .setting-info {
    display: flex;
    align-items: center;
    gap: 16px;
    flex: 1;
  }

  .setting-icon {
    width: 40px;
    height: 40px;
    border-radius: 10px;
    display: grid;
    place-items: center;
    flex-shrink: 0;
  }

  .setting-icon svg {
    width: 20px;
    height: 20px;
    color: white;
  }

  .setting-icon.cloud {
    background: linear-gradient(135deg, #3b82f6, #2563eb);
  }

  .setting-content {
    flex: 1;
  }

  .setting-label {
    margin: 0;
    font-weight: 500;
    color: var(--text);
  }

  .setting-value {
    margin: 2px 0 0 0;
    font-size: 0.9rem;
    color: var(--muted);
  }

  /* Sync Status */
  .status-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 12px;
    margin-bottom: 24px;
  }

  .status-item {
    padding: 12px;
    background: var(--bg-secondary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .status-item .label {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .status-item .value {
    font-weight: 600;
    color: var(--text-primary);
    font-size: 1rem;
  }

  .value.syncing {
    color: var(--accent-color);
  }

  .last-sync {
    margin: 0px 0 24px 0;
    font-size: 0.85rem;
    color: var(--text-secondary);
    text-align: left;
  }

  .sync-message {
    padding: 12px;
    background: color-mix(in srgb, var(--accent) 15%, transparent);
    color: var(--accent-strong);
    border-radius: 8px;
    text-align: center;
    margin-bottom: 24px;
    font-size: 0.9rem;
    border: 1px solid color-mix(in srgb, var(--accent) 25%, transparent);
  }

  .actions-row {
    display: flex;
    gap: 12px;
  }

  /* Devices */
  .device-info {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 24px;
  }

  .device-id {
    padding: 6px 10px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    font-family: "SF Mono", "Monaco", monospace;
    font-size: 0.85rem;
    color: var(--text-primary);
  }

  .divider {
    height: 1px;
    background: var(--border);
    margin: 24px 0;
  }

  .subsection-title {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-secondary);
    margin: 0 0 16px 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .devices-list {
    display: grid;
    gap: 12px;
  }

  .device-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  .device-details {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .device-meta {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .device-actions {
    display: flex;
    gap: 8px;
  }

  /* Versions */
  .search-group {
    margin-bottom: 24px;
  }

  .search-input {
    display: flex;
    gap: 8px;
  }

  .versions-list {
    display: grid;
    gap: 8px;
  }

  .version-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: var(--bg-secondary);
    border-radius: 8px;
    border: 1px solid var(--border);
  }

  .version-info {
    flex: 1;
  }

  .version-header {
    display: flex;
    align-items: baseline;
    gap: 12px;
    margin-bottom: 4px;
  }

  .version-date {
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .version-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .separator {
    color: var(--border);
  }

  .error {
    color: var(--danger);
    margin: 12px 0;
    font-size: 0.9rem;
  }

  .note {
    margin-top: 24px;
    padding: 12px;
    background: var(--bg-secondary);
    border-radius: 8px;
    font-size: 0.85rem;
    color: var(--text-secondary);
    border: 1px solid var(--border);
  }

  .status-pill {
    padding: 6px 10px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--surface-muted);
    color: var(--text-primary);
    font-size: 0.85rem;
    margin-right: 12px;
  }

  .status-pill.offline {
    background: color-mix(in srgb, var(--danger) 15%, transparent);
    border-color: color-mix(in srgb, var(--danger) 30%, transparent);
    color: var(--danger);
  }

  .setting-meta {
    margin: 2px 0 0;
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .mono {
    font-family: "SF Mono", "Monaco", "Courier New", monospace;
  }

  .active-job {
    margin-top: 12px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-secondary);
  }

  .progress-wrapper {
    margin-top: 12px;
    display: grid;
    gap: 6px;
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background: var(--bg-secondary);
    border-radius: 999px;
    overflow: hidden;
    border: 1px solid var(--border);
  }

  .progress-bar.small {
    height: 6px;
  }

  .progress-bar span {
    display: block;
    height: 100%;
    background: var(--accent);
    transition: width 0.2s ease;
  }

  .error-text {
    color: var(--danger);
    margin: 6px 0 0;
    font-size: 0.9rem;
  }

  .loading,
  .empty {
    text-align: center;
    padding: 32px;
    color: var(--text-secondary);
    font-style: italic;
    background: var(--bg-secondary);
    border-radius: 8px;
    border: 1px dashed var(--border);
  }

  @media (max-width: 640px) {
    .settings-container {
      gap: 24px;
    }

    .setting-row {
      padding: 16px;
    }

    .device-item,
    .version-item {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }

    .device-actions {
      width: 100%;
      justify-content: flex-end;
    }
  }
</style>
