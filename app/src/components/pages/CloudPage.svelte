<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import AppHeader from "../layout/AppHeader.svelte";
  import {
    cloudStore,
    isLoggedIn,
    userEmail,
    isSyncing,
  } from "$lib/stores/cloudStore";

  // Login state
  let email = "";
  let password = "";
  let loginError = "";

  function goBack() {
    goto("/settings", { keepFocus: true, noScroll: true });
  }

  // Device management
  let deviceId = "";
  let mockDevices = [
    { id: "current", name: "This Device", last_sync: new Date().toISOString() },
  ];

  // Sync status
  const syncStatusStore = cloudStore.syncStatus;
  const cloudVersionsStore = cloudStore.cloudVersions;

  let syncStatus: any = null;
  let selectedVersions: any[] = [];
  let syncMessage = "";

  // Cloud versions
  let selectedGame = "";
  let loadingVersions = false;

  $: syncStatus = $syncStatusStore;
  $: selectedVersions = $cloudVersionsStore.get(selectedGame) ?? [];

  onMount(async () => {
    // Load initial data if logged in
    if ($isLoggedIn) {
      await loadCloudStatus();
      await loadSyncStatus();
    }
  });

  async function handleLogin() {
    loginError = "";
    const success = await cloudStore.login(email, password);
    if (success) {
      await loadCloudStatus();
      await loadSyncStatus();
    } else {
      loginError = "Invalid credentials";
    }
  }

  async function handleLogout() {
    await cloudStore.logout();
    deviceId = "";
  }

  async function loadCloudStatus() {
    try {
      const status = await cloudStore.getCloudStatus();
      deviceId = status.device_id;
    } catch (error) {
      console.error("Failed to load cloud status:", error);
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
      syncMessage = "Syncing...";
      await cloudStore.forceSyncNow();
      await loadSyncStatus();
      syncMessage = "Sync completed successfully!";
      setTimeout(() => (syncMessage = ""), 3000);
    } catch (error) {
      syncMessage = "Sync failed: " + error;
      setTimeout(() => (syncMessage = ""), 5000);
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

  async function loadCloudVersions() {
    if (!selectedGame) return;

    loadingVersions = true;
    try {
      await cloudStore.listCloudVersions(selectedGame, 10);
    } catch (error) {
      console.error("Failed to load cloud versions:", error);
    } finally {
      loadingVersions = false;
    }
  }

  async function handleDownload(versionId: string) {
    try {
      const path = await cloudStore.downloadCloudVersion(
        selectedGame,
        versionId
      );
      alert(`Downloaded to: ${path}`);
    } catch (error) {
      alert("Download failed: " + error);
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

<section class="cloud-page">
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

      {#if !$isLoggedIn}
        <!-- Login Form -->
        <div class="card login-card">
          <h2>Sign In</h2>
          <p class="subtitle">Connect to CrossSave Cloud to sync your saves</p>

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
              <button type="submit" class="btn-primary btn-large">
                Sign In
              </button>
            </div>
          </form>

          <p class="note">
            <strong>Note:</strong> This is a mock login for development. Any email/password
            will work.
          </p>
        </div>
      {:else}
        <!-- Logged In View -->
        <div class="logged-in-header">
          <div class="user-info">
            <span class="email">{$userEmail}</span>
            <button class="btn-secondary btn-small" on:click={handleLogout}>
              Logout
            </button>
          </div>
        </div>

        <!-- Device Management -->
        <div class="card">
          <h2>Device Management</h2>

          <div class="device-info">
            <div class="info-row">
              <span class="label">Current Device ID:</span>
              <code class="device-id">{deviceId}</code>
            </div>
          </div>

          <div class="devices-list">
            <h3>Registered Devices</h3>
            {#each mockDevices as device}
              <div class="device-item">
                <div class="device-details">
                  <strong>{device.name}</strong>
                  <span class="device-meta"
                    >Last sync: {new Date(
                      device.last_sync
                    ).toLocaleString()}</span
                  >
                </div>
                <div class="device-actions">
                  <button class="btn-secondary btn-small">Rename</button>
                  <button class="btn-danger btn-small">Remove</button>
                </div>
              </div>
            {/each}
          </div>
        </div>

        <!-- Manual Sync Controls -->
        <div class="card">
          <h2>Sync Controls</h2>

          <div class="sync-status">
            {#if syncStatus}
              <div class="status-grid">
                <div class="status-item">
                  <span class="label">Queue Length:</span>
                  <span class="value">{syncStatus.queue_length}</span>
                </div>
                <div class="status-item">
                  <span class="label">Status:</span>
                  <span class="value {$isSyncing ? 'syncing' : 'idle'}">
                    {$isSyncing ? "Syncing..." : "Idle"}
                  </span>
                </div>
                {#if syncStatus.last_sync}
                  <div class="status-item">
                    <span class="label">Last Sync:</span>
                    <span class="value"
                      >{new Date(syncStatus.last_sync).toLocaleString()}</span
                    >
                  </div>
                {/if}
              </div>
            {/if}

            {#if syncMessage}
              <p class="sync-message">{syncMessage}</p>
            {/if}
          </div>

          <div class="sync-actions">
            <button
              class="btn-primary btn-large"
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

        <!-- Cloud Version History -->
        <div class="card">
          <h2>Cloud Version History</h2>

          <div class="form-group">
            <label for="game-select">Select Game:</label>
            <input
              id="game-select"
              type="text"
              bind:value={selectedGame}
              placeholder="Enter game ID"
              on:blur={loadCloudVersions}
            />
          </div>

          {#if loadingVersions}
            <p class="loading">Loading versions...</p>
          {:else if selectedVersions.length > 0}
            <div class="versions-list">
              {#each selectedVersions as version}
                <div class="version-item">
                  <div class="version-info">
                    <div class="version-header">
                      <strong>{version.version_id.substring(0, 8)}...</strong>
                      <span class="version-date"
                        >{formatDate(version.timestamp)}</span
                      >
                    </div>
                    <div class="version-meta">
                      <span>Size: {formatBytes(version.size_bytes)}</span>
                      <span>Device: {version.device_id.substring(0, 8)}...</span
                      >
                    </div>
                  </div>
                  <div class="version-actions">
                    <button
                      class="btn-secondary btn-small"
                      on:click={() => handleDownload(version.version_id)}
                    >
                      Download
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {:else if selectedGame}
            <p class="empty">No cloud versions found for this game</p>
          {/if}
        </div>
      {/if}
    </main>
  </div>
</section>

<style>
  .cloud-page {
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

  h2 {
    margin: 0 0 1rem 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  h3 {
    margin: 1.5rem 0 1rem 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 24px;
    margin-bottom: 24px;
    box-shadow: var(--shadow-soft);
    overflow: hidden;
    display: grid;
    gap: 16px;
  }

  .subtitle {
    color: var(--text-secondary);
    margin-bottom: 2rem;
  }

  .input-stack {
    display: grid;
    gap: 12px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: color-mix(in srgb, var(--surface-muted) 70%, transparent);
    box-shadow: inset 0 1px 0 color-mix(in srgb, var(--surface) 40%, transparent);
  }

  .form-group {
    display: grid;
    gap: 8px;
    padding: 12px 14px;
    background: color-mix(in srgb, var(--surface) 85%, transparent);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .input-stack .form-group {
    border: 0;
    padding: 0;
    background: transparent;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .form-actions {
    margin-top: 12px;
    display: flex;
    justify-content: center;
    width: 100%;
  }

  input[type="email"],
  input[type="password"],
  input[type="text"] {
    width: 100%;
    padding: 0.75rem 1rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--surface-muted) 85%, transparent);
    color: var(--text-primary);
    font-size: 1rem;
    transition: border-color 0.2s ease, box-shadow 0.2s ease,
      background-color 0.2s ease;
  }

  input:focus {
    outline: none;
    border-color: var(--accent-color);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-color) 22%, transparent);
    background: var(--surface);
  }

  .btn-primary,
  .btn-secondary,
  .btn-danger {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 8px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-large {
    width: min(320px, 100%);
    padding: 12px;
    font-size: 1rem;
  }

  .btn-small {
    padding: 0.5rem 1rem;
    font-size: 0.9rem;
  }

  .btn-primary {
    background: var(--accent);
    color: #fff;
    border: 1px solid color-mix(in srgb, var(--accent) 88%, transparent);
    box-shadow: 0 10px 22px color-mix(in srgb, var(--accent) 26%, transparent);
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent-strong);
    border-color: color-mix(in srgb, var(--accent-strong) 94%, transparent);
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
    background: var(--danger);
    color: white;
  }

  .btn-danger:hover {
    background: var(--danger-dark);
  }

  .btn-primary:disabled,
  .btn-secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error {
    color: #ef4444;
    margin: 1rem 0;
  }

  .note {
    margin-top: 1.5rem;
    padding: 1rem;
    background: var(--bg-secondary);
    border-radius: 8px;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .logged-in-header {
    margin-bottom: 2rem;
  }

  .user-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background: var(--surface-muted);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }

  .email {
    font-weight: 500;
    color: var(--text-primary);
  }

  .device-info {
    margin-bottom: 2rem;
  }

  .info-row {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem;
    background: var(--surface-muted);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }

  .label {
    font-weight: 500;
    color: var(--text-secondary);
  }

  .device-id {
    flex: 1;
    padding: 0.5rem;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font-family: "SF Mono", "Monaco", monospace;
    font-size: 0.85rem;
    color: var(--text-primary);
  }

  .devices-list {
    margin-top: 1.5rem;
    display: grid;
    gap: 12px;
  }

  .device-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background: var(--surface-muted);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .device-details {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .device-meta {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .device-actions {
    display: flex;
    gap: 0.5rem;
  }

  .sync-status {
    margin-bottom: 1.5rem;
  }

  .status-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 1rem;
    background: color-mix(in srgb, var(--surface-muted) 60%, transparent);
    padding: 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }

  .status-item {
    padding: 1rem;
    background: var(--surface);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .value {
    font-weight: 600;
    color: var(--text-primary);
  }

  .value.syncing {
    color: var(--accent-color);
  }

  .value.idle {
    color: var(--text-secondary);
  }

  .sync-message {
    padding: 0.75rem;
    background: var(--accent-color);
    color: white;
    border-radius: 8px;
    text-align: center;
  }

  .sync-actions {
    display: flex;
    gap: 1rem;
  }

  .sync-actions .btn-large {
    width: 100%;
    flex: 2;
  }

  .sync-actions .btn-secondary {
    flex: 1;
  }

  .loading,
  .empty {
    text-align: center;
    padding: 2rem;
    color: var(--text-secondary);
    font-style: italic;
  }

  .versions-list {
    margin-top: 1rem;
    display: grid;
    gap: 12px;
  }

  .version-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background: var(--surface-muted);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }

  .version-info {
    flex: 1;
  }

  .version-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
  }

  .version-date {
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .version-meta {
    display: flex;
    gap: 1.5rem;
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  @media (max-width: 640px) {
    .card {
      padding: 1.5rem;
    }

    .sync-actions {
      flex-direction: column;
    }

    .device-item,
    .version-item {
      flex-direction: column;
      align-items: flex-start;
      gap: 1rem;
    }

    .device-actions,
    .version-actions {
      width: 100%;
    }

    .device-actions button,
    .version-actions button {
      flex: 1;
    }
  }
</style>
