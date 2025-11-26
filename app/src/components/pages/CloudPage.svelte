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
    connectionStatusStore,
  } from "$lib/stores/cloudStore";
  import type {
    CloudDevice,
    SyncStatus,
    CloudVersion,
  } from "$lib/stores/cloudStore";
  import { pushError, pushInfo, pushSuccess } from "$lib/notifications";
  import { formatErrorMessage } from "$lib/errorMessages";

  // Login state
  let email = "";
  let password = "";
  let confirmPassword = "";
  let loginError = "";
  let passwordErrors: string[] = [];
  let isSignupMode = false;
  let rememberLogin = false;
  let isLoggingIn = false;

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
    // Load saved credentials if remember login was checked
    const savedEmail = localStorage.getItem("cloud:savedEmail");
    const savedPassword = localStorage.getItem("cloud:savedPassword");
    const savedRemember = localStorage.getItem("cloud:rememberLogin");

    if (savedEmail && savedPassword && savedRemember === "true") {
      email = savedEmail;
      password = savedPassword;
      rememberLogin = true;
    }

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

  // Password validation
  function validatePassword(pwd: string): string[] {
    const errors: string[] = [];

    if (pwd.length < 8) {
      errors.push("At least 8 characters");
    }
    if (!/[A-Z]/.test(pwd)) {
      errors.push("At least one uppercase letter");
    }
    if (!/[a-z]/.test(pwd)) {
      errors.push("At least one lowercase letter");
    }
    if (!/[0-9]/.test(pwd)) {
      errors.push("At least one number");
    }
    if (!/[!@#$%^&*(),.?":{}|<>]/.test(pwd)) {
      errors.push("At least one special character");
    }

    return errors;
  }

  // Update password errors when password changes in signup mode
  $: if (isSignupMode && password) {
    passwordErrors = validatePassword(password);
  } else {
    passwordErrors = [];
  }

  // Computed password validation checks for UI
  $: hasMinLength = password.length >= 8;
  $: hasUppercase = /[A-Z]/.test(password);
  $: hasLowercase = /[a-z]/.test(password);
  $: hasNumber = /[0-9]/.test(password);
  $: hasSpecialChar = /[!@#$%^&*(),.?":{}|<>]/.test(password);

  async function handleLogin() {
    loginError = "";
    isLoggingIn = true;

    try {
      const result = await cloudStore.login(email, password);
      if (result.success) {
        // Save credentials if remember login is checked
        if (rememberLogin) {
          localStorage.setItem("cloud:savedEmail", email);
          localStorage.setItem("cloud:savedPassword", password);
          localStorage.setItem("cloud:rememberLogin", "true");
        } else {
          localStorage.removeItem("cloud:savedEmail");
          localStorage.removeItem("cloud:savedPassword");
          localStorage.removeItem("cloud:rememberLogin");
        }

        await initializeAfterLogin();
        pushSuccess("Logged in successfully");
      } else {
        loginError = formatErrorMessage(result.error ?? "Invalid credentials");
      }
    } finally {
      isLoggingIn = false;
    }
  }

  async function handleSignup() {
    loginError = "";

    // Validate password
    const errors = validatePassword(password);
    if (errors.length > 0) {
      loginError = "Password does not meet requirements";
      return;
    }

    // Check password confirmation
    if (password !== confirmPassword) {
      loginError = "Passwords do not match";
      return;
    }

    isLoggingIn = true;

    try {
      const result = await cloudStore.signup(email, password);
      if (result.success) {
        // Save credentials if remember login is checked
        if (rememberLogin) {
          localStorage.setItem("cloud:savedEmail", email);
          localStorage.setItem("cloud:savedPassword", password);
          localStorage.setItem("cloud:rememberLogin", "true");
        }

        await initializeAfterLogin();
        pushSuccess("Account created successfully");
      } else {
        loginError = formatErrorMessage(result.error ?? "Signup failed");
      }
    } finally {
      isLoggingIn = false;
    }
  }

  async function handleLogout() {
    await cloudStore.logout();
    deviceId = "";
    devices = [];

    // Clear saved credentials on logout
    localStorage.removeItem("cloud:savedEmail");
    localStorage.removeItem("cloud:savedPassword");
    localStorage.removeItem("cloud:rememberLogin");

    email = "";
    password = "";
    rememberLogin = false;

    pushInfo("Logged out successfully");
  }

  async function loadCloudStatus() {
    try {
      const status = await cloudStore.getCloudStatus();
      deviceId = status.device_id;
      onlineStatus = status.connected ? "online" : "offline";
    } catch (error) {
      loginError = formatErrorMessage(error);
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
    if (!confirm(`Remove device ${device.device_id}?`)) return;
    try {
      await cloudStore.removeDevice(device.device_id);
      pushSuccess("Device removed");
    } catch (error) {
      pushError(formatErrorMessage(error));
    }
  }

  async function loadCloudVersions() {
    if (!selectedGame) return;

    loadingVersions = true;
    try {
      await cloudStore.listCloudVersions(selectedGame);
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
      pushError(formatErrorMessage(error));
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

      <!-- Connection Status Banner -->
      <!-- Only show if we've checked connection and it's offline -->
      {#if $connectionStatusStore.last_success !== undefined && !$connectionStatusStore.connected}
        <div class="connection-banner offline">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path
              d="M1 1l22 22M16.72 11.06A10.94 10.94 0 0 1 19 12.55M5 12.55a10.94 10.94 0 0 1 5.17-2.39M10.71 5.05A16 16 0 0 1 22.58 9M1.42 9a15.91 15.91 0 0 1 4.7-2.88M8.53 16.11a6 6 0 0 1 6.95 0M12 20h.01"
            ></path>
          </svg>
          <span>Offline - Checking connection...</span>
        </div>
      {/if}

      <div class="settings-container">
        {#if !$isLoggedIn}
          <!-- Login Form -->
          <div class="section-group">
            <div class="section-header">
              <p class="section-title">
                {isSignupMode ? "Sign Up" : "Sign In"}
              </p>
            </div>
            <div class="settings-card">
              <div class="card-content">
                <p class="description">
                  {isSignupMode
                    ? "Create an account to sync your saves across devices"
                    : "Connect to CrossSave Cloud to sync your saves"}
                </p>

                <form
                  on:submit|preventDefault={isSignupMode
                    ? handleSignup
                    : handleLogin}
                >
                  <div class="input-stack">
                    <div class="form-group">
                      <label for="email">Email</label>
                      <input
                        id="email"
                        type="email"
                        bind:value={email}
                        placeholder="your@email.com"
                        required
                        disabled={isLoggingIn}
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
                        disabled={isLoggingIn}
                      />
                    </div>

                    {#if isSignupMode}
                      <div class="form-group">
                        <label for="confirm-password">Confirm Password</label>
                        <input
                          id="confirm-password"
                          type="password"
                          bind:value={confirmPassword}
                          placeholder="••••••••"
                          required
                          disabled={isLoggingIn}
                        />
                      </div>

                      {#if password}
                        <div class="password-requirements">
                          <p class="requirements-title">Password must have:</p>
                          <ul class="requirements-list">
                            <li class:valid={hasMinLength}>
                              <span class="check-icon"
                                >{hasMinLength ? "✓" : "○"}</span
                              >
                              At least 8 characters
                            </li>
                            <li class:valid={hasUppercase}>
                              <span class="check-icon"
                                >{hasUppercase ? "✓" : "○"}</span
                              >
                              One uppercase letter
                            </li>
                            <li class:valid={hasLowercase}>
                              <span class="check-icon"
                                >{hasLowercase ? "✓" : "○"}</span
                              >
                              One lowercase letter
                            </li>
                            <li class:valid={hasNumber}>
                              <span class="check-icon"
                                >{hasNumber ? "✓" : "○"}</span
                              >
                              One number
                            </li>
                            <li class:valid={hasSpecialChar}>
                              <span class="check-icon"
                                >{hasSpecialChar ? "✓" : "○"}</span
                              >
                              One special character
                            </li>
                          </ul>
                        </div>
                      {/if}
                    {/if}

                    <div class="form-group checkbox-group">
                      <label class="checkbox-label">
                        <input
                          type="checkbox"
                          bind:checked={rememberLogin}
                          disabled={isLoggingIn}
                        />
                        <span>Remember login</span>
                      </label>
                    </div>
                  </div>

                  {#if loginError}
                    <p class="error">{loginError}</p>
                  {/if}

                  <div class="form-actions">
                    <button
                      type="submit"
                      class="btn-primary btn-full"
                      disabled={isLoggingIn}
                    >
                      {isLoggingIn
                        ? "Please wait..."
                        : isSignupMode
                          ? "Create Account"
                          : "Sign In"}
                    </button>

                    <button
                      type="button"
                      class="btn-secondary btn-full"
                      on:click={() => {
                        isSignupMode = !isSignupMode;
                        loginError = "";
                      }}
                      disabled={isLoggingIn}
                    >
                      {isSignupMode
                        ? "Already have an account? Sign In"
                        : "Need an account? Sign Up"}
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
              <div class="setting-row account-row">
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
                    <div class="account-main-row">
                      <div class="account-info-group">
                        <p class="setting-label">Logged in as</p>
                        <div class="setting-value-row">
                          <p class="setting-value">
                            {$userEmail || "Cloud account"}
                          </p>
                          <span
                            class="status-dot"
                            class:offline={onlineStatus === "offline"}
                            aria-live="polite"
                            title={onlineStatus === "online"
                              ? "Online"
                              : "Offline"}
                          ></span>
                        </div>
                      </div>
                      <button
                        class="btn-secondary btn-small"
                        on:click={handleLogout}
                      >
                        Logout
                      </button>
                    </div>
                    <p class="setting-meta">
                      Device ID: {deviceId || "Unknown"}
                    </p>
                  </div>
                </div>
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
                        syncStatus.last_sync
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
                          <strong>{device.platform || "unknown"}</strong>
                          <span class="device-meta"
                            >Last seen: {new Date(
                              (device.last_seen || 0) * 1000
                            ).toLocaleString()}</span
                          >
                          <span class="device-meta mono"
                            >ID: {device.device_id.toLocaleString()}</span
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
                            <span>{formatBytes(version.size_bytes)}</span>
                            <span class="separator">•</span>
                            <span class="mono"
                              >Hash: {version.sha256.substring(0, 10)}...</span
                            >
                          </div>
                          <div class="version-meta">
                            <span
                              >Device: {version.device_id.substring(
                                0,
                                8
                              )}...</span
                            >
                            {#if version.device_id === deviceId}
                              <span class="tag">This device</span>
                            {/if}
                            <span class="separator">•</span>
                            <span>Files: {version.file_list.length}</span>
                          </div>
                          {#if downloadState.versionId === version.version_id}
                            <div class="progress-bar small">
                              <span style={`width: ${downloadState.progress}%`}
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
  }

  .content-body {
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

  .checkbox-group {
    padding-top: 4px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    font-weight: 400;
  }

  .checkbox-label input[type="checkbox"] {
    width: 18px;
    height: 18px;
    cursor: pointer;
    accent-color: var(--accent);
  }

  .checkbox-label span {
    user-select: none;
  }

  .password-requirements {
    padding: 12px;
    background: color-mix(in srgb, var(--surface-muted) 50%, transparent);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    margin-top: 8px;
  }

  .requirements-title {
    margin: 0 0 8px 0;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .requirements-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    gap: 6px;
  }

  .requirements-list li {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.85rem;
    color: var(--text-secondary);
    transition: color 0.2s;
  }

  .requirements-list li.valid {
    color: var(--success, #10b981);
  }

  .requirements-list li.valid .check-icon {
    color: var(--success, #10b981);
    font-weight: bold;
  }

  .check-icon {
    font-size: 1rem;
    width: 16px;
    text-align: center;
    color: var(--text-muted);
  }

  .form-actions {
    margin-top: 12px;
    display: grid;
    gap: 10px;
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

  .setting-value-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .account-row .setting-info {
    flex: 1;
    max-width: 100%;
  }

  .account-main-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 12px;
  }

  .account-info-group {
    display: flex;
    flex-direction: column;
    gap: 0px;
    flex: 1;
  }

  .account-info-group .setting-value {
    margin-top: 0;
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
    grid-template-columns: repeat(3, 1fr);
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
    text-align: center;
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

  .tag {
    margin-left: 8px;
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--primary, #4f46e5);
    color: white;
    font-size: 0.75rem;
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
    white-space: nowrap;
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

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--success, #10b981);
    flex-shrink: 0;
    box-shadow: 0 0 0 2px
      color-mix(in srgb, var(--success, #10b981) 20%, transparent);
    animation: pulse 2s ease-in-out infinite;
  }

  .status-dot.offline {
    background: var(--danger, #ef4444);
    box-shadow: 0 0 0 2px
      color-mix(in srgb, var(--danger, #ef4444) 20%, transparent);
    animation: none;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.6;
    }
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

  .connection-banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    margin-bottom: 16px;
    border-radius: 8px;
    font-size: 0.9rem;
    font-weight: 500;
    animation: fadeIn 0.3s ease;
  }

  .connection-banner.offline {
    background: color-mix(in srgb, var(--warning) 10%, transparent);
    color: var(--warning);
    border: 1px solid color-mix(in srgb, var(--warning) 20%, transparent);
  }

  .connection-banner svg {
    flex-shrink: 0;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (max-width: 640px) {
    .settings-container {
      gap: 24px;
    }

    .setting-row {
      padding: 16px;
    }

    .status-grid {
      grid-template-columns: 1fr;
    }

    .device-info {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }

    .device-id {
      width: 100%;
      text-align: center;
      display: block;
    }

    .device-item,
    .version-item {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }

    .device-actions {
      width: 100%;
      justify-content: center;
    }
  }
</style>
