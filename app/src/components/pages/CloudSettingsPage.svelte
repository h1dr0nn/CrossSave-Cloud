<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import AppHeader from "../layout/AppHeader.svelte";
  import { pushInfo, pushSuccess, pushError } from "$lib/notifications";
  import {
    cloudStore,
    type CloudAuthMode,
    type CloudConfig,
    type CloudMode,
    type CloudValidationResult,
    type SelfHostSettings,
  } from "$lib/stores/cloudStore";

  const modeStore = cloudStore.cloudMode;
  const configStore = cloudStore.cloudConfig;
  const validationStore = cloudStore.validation;
  const onlineStatusStore = cloudStore.onlineStatus;

  const modes: { label: string; value: CloudMode }[] = [
    { label: "Sync Off", value: "off" },
    { label: "Sync On", value: "official" },
    { label: "Self-host", value: "self_host" },
  ];

  const authOptions: { label: string; value: CloudAuthMode }[] = [
    { label: "NONE", value: "NONE" },
    { label: "ACCESS_KEY", value: "ACCESS_KEY" },
    { label: "USERPASS", value: "USERPASS" },
  ];

  let activeMode: CloudMode = "off";
  let onlineStatus: "online" | "offline" | "fail" | "ready" = "offline";
  let localConfig: CloudConfig = { mode: "off", auth_mode: "NONE" };
  let validation: CloudValidationResult = { status: "idle", message: "" };
  let connectionStatus: "online" | "offline" = "offline";
  let loading = true;
  let saving = false;
  let validating = false;
  let statusMessage = "";

  // Self-host specific fields
  let idServer = "";
  let relayServer = "";
  let apiServer = "";
  let key = "";
  let connecting = false;
  let officialConnecting = false;
  let officialConnectionFailed = false;
  let unlisteners: UnlistenFn[] = [];

  $: activeMode = ($modeStore as CloudMode) ?? activeMode;
  $: connectionStatus = ($onlineStatusStore as "online" | "offline") ?? connectionStatus;
  $: validation = ($validationStore as CloudValidationResult) ?? {
    status: "idle",
    message: "",
  };
  $: {
    if (validation.status === "valid") {
      onlineStatus = "ready";
      officialConnecting = false;
      connecting = false;
      officialConnectionFailed = false;
      statusMessage = validation.message ?? statusMessage;
    } else if (validation.status === "invalid") {
      onlineStatus = "fail";
      officialConnecting = false;
      connecting = false;
      officialConnectionFailed = activeMode === "official";
      statusMessage = validation.message ?? statusMessage;
    } else {
      onlineStatus = connectionStatus ?? "offline";
    }
  }
  $: if ($configStore) {
    localConfig = { ...localConfig, ...$configStore };
    if ($configStore.self_host) {
      idServer = $configStore.self_host.id_server ?? idServer;
      relayServer = $configStore.self_host.relay_server ?? relayServer;
      apiServer = $configStore.self_host.api_server ?? apiServer;
      key = $configStore.self_host.access_key ?? key;
    }
  }

  function currentSelfHostSettings(): SelfHostSettings {
    return {
      id_server: idServer.trim(),
      relay_server: relayServer.trim(),
      api_server: apiServer.trim(),
      access_key: key.trim(),
    };
  }

  async function setupEventListeners() {
    try {
      unlisteners = await Promise.all([
        listen("sync://online", () => {
          connectionStatus = "online";
          if (validation.status === "idle") {
            onlineStatus = "online";
          }
        }),
        listen("sync://offline", () => {
          connectionStatus = "offline";
          if (validation.status === "idle") {
            onlineStatus = "offline";
          }
        }),
        listen<{ mode: CloudMode; config?: CloudConfig }>(
          "cloud://mode-changed",
          (event) => {
            activeMode = event.payload.mode;
            if (event.payload.config) {
              localConfig = { ...localConfig, ...event.payload.config };
            }
          }
        ),
        listen<{ config: CloudConfig }>("cloud://backend-switched", (event) => {
          localConfig = { ...localConfig, ...event.payload.config };
          if (event.payload.config?.mode) {
            activeMode = event.payload.config.mode as CloudMode;
          }
        }),
        listen<{ mode: CloudMode; message: string }>(
          "cloud://config-valid",
          (event) => {
            statusMessage = event.payload?.message ?? statusMessage;
            officialConnectionFailed = false;
          }
        ),
        listen<{ mode: CloudMode; message: string }>(
          "cloud://config-invalid",
          (event) => {
            statusMessage = event.payload?.message ?? statusMessage;
            officialConnectionFailed = event.payload?.mode === "official";
            connecting = false;
            officialConnecting = false;
          }
        ),
        listen("sync://download-progress", (event) => {
          console.debug("[CloudSettings] Download progress", event.payload);
        }),
        listen("sync://download-complete", (event) => {
          console.debug("[CloudSettings] Download complete", event.payload);
        }),
        listen("sync://download-error", (event) => {
          console.debug("[CloudSettings] Download error", event.payload);
        }),
      ]);
    } catch (error) {
      console.error("Failed to bind cloud events", error);
    }
  }

  onMount(async () => {
    await setupEventListeners();
    try {
      await cloudStore.initialize();
      const config = (await cloudStore.getCloudConfig()) ?? {};
      localConfig = { ...localConfig, ...config };
      activeMode = (config.mode as CloudMode) ?? activeMode;
    } catch (error) {
      console.error("Failed to initialize cloud settings:", error);
    } finally {
      loading = false;
    }
  });

  onDestroy(() => {
    unlisteners.forEach((fn) => fn());
    unlisteners = [];
  });

  function goBack() {
    goto("/settings", { keepFocus: true, noScroll: true });
  }

  async function handleModeChange(newMode: CloudMode) {
    // Don't do anything if already on this mode
    if (newMode === activeMode) return;

    if (saving) return;

    console.log(`[CloudSettings] Switching to mode: ${newMode}`);

    // Update UI immediately (optimistic update)
    activeMode = newMode;
    statusMessage = "";
    officialConnectionFailed = false;
    localConfig = { ...localConfig, mode: newMode };

    if (newMode === "official") {
      officialConnecting = true;
    } else if (newMode === "self_host") {
      connecting = true;
    } else {
      officialConnecting = false;
      connecting = false;
    }

    // Update backend in background
    saving = true;
    try {
      await cloudStore.updateCloudMode(newMode);
      if (newMode === "official") {
        void cloudStore
          .validateOfficialCloudSettings(localConfig)
          .finally(() => {
            officialConnecting = false;
          });
      } else if (newMode === "self_host") {
        const selfHost = currentSelfHostSettings();
        localConfig = { ...localConfig, self_host: selfHost, mode: "self_host" };
        await cloudStore.updateCloudSettings(localConfig);
        void cloudStore
          .validateSelfHostSettings({ ...localConfig, self_host: selfHost, mode: "self_host" })
          .finally(() => {
            connecting = false;
          });
      }
    } catch (error) {
      console.error(
        "[CloudSettings] Failed to update cloud mode in backend:",
        error
      );
    } finally {
      saving = false;
    }
  }

  async function handleReconnect() {
    officialConnecting = true;
    statusMessage = "";
    try {
      await cloudStore.validateOfficialCloudSettings(localConfig);
    } catch (error) {
      pushError(
        typeof error === "string"
          ? error
          : ((error as Error)?.message ?? "Validation failed")
      );
    } finally {
      officialConnecting = false;
    }
  }

  async function handleCopyConfig() {
    const config = `${idServer}|${relayServer}|${apiServer}|${key}`;
    try {
      await navigator.clipboard.writeText(config);
      pushInfo("Configuration copied to clipboard");
    } catch (error) {
      pushError("Failed to copy configuration");
    }
  }

  async function handlePasteConfig() {
    try {
      const text = await navigator.clipboard.readText();
      const parts = text.split("|");
      if (parts.length === 4) {
        idServer = parts[0].trim();
        relayServer = parts[1].trim();
        apiServer = parts[2].trim();
        key = parts[3].trim();
        pushSuccess("Configuration pasted successfully");
      } else {
        pushError("Invalid configuration format");
      }
    } catch (error) {
      pushError("Failed to paste configuration");
    }
  }

  async function handleSaveSelfHost() {
    if (!idServer || !relayServer || !apiServer || !key) {
      pushError("Please fill in all fields");
      return;
    }

    saving = true;
    connecting = true;
    statusMessage = "";
    const selfHost = currentSelfHostSettings();
    localConfig = { ...localConfig, self_host: selfHost, mode: "self_host" };

    try {
      await cloudStore.updateCloudSettings(localConfig);
      await cloudStore.validateSelfHostSettings(localConfig);
      pushSuccess("Configuration saved");
    } catch (error) {
      statusMessage =
        typeof error === "string"
          ? error
          : ((error as Error)?.message ?? "Connection failed");
      pushError("Connection failed: " + statusMessage);
    } finally {
      saving = false;
      connecting = false;
    }
  }

  function handleClearSelfHost() {
    idServer = "";
    relayServer = "";
    apiServer = "";
    key = "";
    pushInfo("Configuration cleared");
  }

  async function updateOfficialField(field: keyof CloudConfig, value: string) {
    if (activeMode !== "official") return;
    saving = true;
    const updated = { ...localConfig, [field]: value, mode: "official" };
    localConfig = updated;
    try {
      await cloudStore.updateCloudSettings(updated);
    } catch (error) {
      pushError(
        typeof error === "string"
          ? error
          : ((error as Error)?.message ?? "Failed to update settings")
      );
    } finally {
      saving = false;
    }
  }

  async function validateOfficial() {
    validating = true;
    try {
      statusMessage = "";
      await cloudStore.validateOfficialCloudSettings(localConfig);
    } catch (error) {
      pushError(
        typeof error === "string"
          ? error
          : ((error as Error)?.message ?? "Validation failed")
      );
    } finally {
      validating = false;
    }
  }
</script>

<section class="settings-page">
  <div class="content-surface">
    <main class="content-body">
      <div class="header-wrapper">
        <AppHeader
          title="Cloud Mode"
          showBack
          onBack={goBack}
          onMenu={() => {}}
          sticky={false}
        />
      </div>

      <div class="settings-container">
        <div class="section-group">
          <div class="tab-shell">
            <div class="tab-header">
              <div class="segmented nowrap" role="tablist" aria-label="Cloud mode">
                {#each modes as mode}
                  <button
                    type="button"
                    role="tab"
                    aria-pressed={activeMode === mode.value}
                    class:active={activeMode === mode.value}
                    on:click={() => handleModeChange(mode.value)}
                  >
                    {mode.label}
                  </button>
                {/each}
              </div>

              <div class="status-chip" aria-live="polite">
                <span class={`dot ${onlineStatus}`}></span>
                <span class="status-text">
                  {#if onlineStatus === "online"}
                    Online
                  {:else if onlineStatus === "ready"}
                    Ready
                  {:else if onlineStatus === "fail"}
                    Failed
                  {:else}
                    Offline
                  {/if}
                </span>
              </div>
            </div>

            {#if loading}
              <div class="loading">Loading cloud settings...</div>
            {:else if activeMode === "off"}
              <div class="disabled-card">
                <p class="disabled-label">Cloud Sync Disabled</p>
                <p class="muted">
                  Switch to Official Cloud or Self-host to configure sync.
                </p>
              </div>
            {:else if activeMode === "official"}
              <div class="disabled-card">
                <p class="disabled-label">Cloud Sync Enabled</p>
                <p class="muted">
                  Your saves are automatically syncing to the official cloud.
                </p>

                {#if officialConnecting}
                  <div class="connection-status">
                    <div class="loading-spinner"></div>
                    <p class="connection-text">Connecting to cloud server...</p>
                  </div>
                {:else if officialConnectionFailed}
                  <div class="connection-status failed">
                    <p class="connection-text error">Connection failed</p>
                    <button class="btn-primary" on:click={handleReconnect}>
                      Reconnect
                    </button>
                  </div>
                {/if}
              </div>
            {:else if activeMode === "self_host"}
              <div class="settings-card">
                <div class="card-simple-header">
                  <p class="card-simple-title">ID/Relay Server</p>
                  <div class="header-actions">
                    <button
                      class="btn-icon-only"
                      on:click={handleCopyConfig}
                      title="Copy configuration"
                    >
                      <svg viewBox="0 0 24 24" fill="none" class="icon">
                        <rect
                          x="9"
                          y="9"
                          width="13"
                          height="13"
                          rx="2"
                          stroke="currentColor"
                          stroke-width="2"
                        />
                        <path
                          d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"
                          stroke="currentColor"
                          stroke-width="2"
                        />
                      </svg>
                    </button>
                    <button
                      class="btn-icon-only"
                      on:click={handlePasteConfig}
                      title="Paste configuration"
                    >
                      <svg viewBox="0 0 24 24" fill="none" class="icon">
                        <path
                          d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"
                          stroke="currentColor"
                          stroke-width="2"
                        />
                        <rect
                          x="8"
                          y="2"
                          width="8"
                          height="4"
                          rx="1"
                          stroke="currentColor"
                          stroke-width="2"
                        />
                      </svg>
                    </button>
                  </div>
                </div>

                <div class="card-separator"></div>

                <div class="card-content">
                  <div class="input-row">
                    <label for="id-server">ID Server</label>
                    <input
                      id="id-server"
                      type="text"
                      placeholder="id.server.com"
                      bind:value={idServer}
                    />
                  </div>

                  <div class="input-row">
                    <label for="relay-server">Relay Server</label>
                    <input
                      id="relay-server"
                      type="text"
                      placeholder="relay.server.com"
                      bind:value={relayServer}
                    />
                  </div>

                  <div class="input-row">
                    <label for="api-server">API Server</label>
                    <input
                      id="api-server"
                      type="text"
                      placeholder="api.server.com"
                      bind:value={apiServer}
                    />
                  </div>

                  <div class="input-row">
                    <label for="key">Key</label>
                    <input
                      id="key"
                      type="password"
                      placeholder="Enter your key"
                      bind:value={key}
                    />
                  </div>

                  <div class="button-row">
                    <button
                      class="btn-primary"
                      on:click={handleSaveSelfHost}
                      disabled={saving || connecting}
                    >
                      {connecting ? "Connecting..." : "Save & Connect"}
                    </button>
                    <button class="btn-danger" on:click={handleClearSelfHost}>
                      Clear
                    </button>
                  </div>
                </div>
              </div>
            {/if}
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

  .tab-shell {
    display: grid;
    gap: 16px;
  }

  .tab-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }

  .segmented {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    background: color-mix(in srgb, var(--surface) 80%, transparent);
    border: 1px solid var(--border);
    border-radius: 14px;
    padding: 4px;
    gap: 6px;
  }

  .segmented.nowrap {
    white-space: nowrap;
  }

  .segmented button {
    border: none;
    background: transparent;
    color: var(--text);
    padding: 10px 12px;
    border-radius: 10px;
    font-weight: 600;
    cursor: pointer;
    transition:
      background 0.2s,
      color 0.2s,
      transform 0.1s ease;
  }

  .segmented button.active,
  .segmented button[aria-pressed="true"] {
    background: linear-gradient(135deg, #4f46e5, #7c3aed);
    color: white;
    box-shadow: 0 6px 18px rgba(79, 70, 229, 0.35);
  }

  .segmented button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .segmented.auth {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .status-chip {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: color-mix(in srgb, var(--surface) 90%, transparent);
    font-weight: 600;
  }

  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    display: inline-block;
  }

  .dot.online {
    background: #10b981;
  }

  .dot.offline {
    background: #6b7280;
  }

  .dot.fail {
    background: #ef4444;
  }

  .dot.ready {
    background: #3b82f6;
  }

  .settings-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--shadow-soft);
    overflow: hidden;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 18px;
    border-bottom: 1px solid var(--border);
    background: color-mix(in srgb, var(--surface) 95%, var(--text) 5%);
  }

  .card-simple-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 18px;
  }

  .card-title {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
    color: var(--text);
  }

  .card-simple-title {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 700;
    color: var(--text);
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .card-separator {
    height: 1px;
    background: var(--border);
    margin: 0 18px;
  }

  .card-content {
    padding: 16px 18px;
    display: grid;
    gap: 16px;
  }

  .input-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .input-row {
    display: grid;
    grid-template-columns: 140px 1fr;
    align-items: center;
    gap: 12px;
  }

  .input-row label {
    margin: 0;
    font-weight: 600;
    color: var(--text);
  }

  label {
    font-weight: 600;
    color: var(--text-primary, var(--text));
  }

  input {
    width: 100%;
    padding: 12px;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: var(--surface-muted, rgba(255, 255, 255, 0.02));
    color: var(--text);
    font-size: 1rem;
  }

  .actions-row {
    display: flex;
    justify-content: flex-end;
  }

  .button-row {
    display: flex;
    gap: 8px;
    justify-content: stretch;
  }

  .button-row button {
    flex: 1;
  }

  .btn-primary {
    border: none;
    background: linear-gradient(135deg, #2563eb, #1d4ed8);
    color: white;
    padding: 12px 18px;
    border-radius: 12px;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 8px 20px rgba(37, 99, 235, 0.25);
  }

  .btn-primary:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .btn-secondary {
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    padding: 12px 18px;
    border-radius: 12px;
    font-weight: 600;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    transition: all 0.2s;
  }

  .btn-secondary:hover {
    background: var(--surface-muted);
  }

  .btn-danger {
    border: none;
    background: linear-gradient(135deg, #ef4444, #dc2626);
    color: white;
    padding: 12px 18px;
    border-radius: 12px;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 4px 12px rgba(239, 68, 68, 0.25);
    transition: all 0.2s;
  }

  .btn-danger:hover {
    background: linear-gradient(135deg, #dc2626, #b91c1c);
  }

  .btn-icon {
    width: 16px;
    height: 16px;
  }

  .btn-icon-only {
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text);
    padding: 8px;
    border-radius: 8px;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .btn-icon-only:hover {
    background: var(--surface-muted);
    border-color: var(--text-muted);
  }

  .btn-icon-only .icon {
    width: 18px;
    height: 18px;
  }

  .validation {
    margin: 0;
    padding: 12px;
    border-radius: 10px;
    font-weight: 600;
  }

  .validation.valid {
    background: rgba(16, 185, 129, 0.12);
    color: #065f46;
    border: 1px solid rgba(16, 185, 129, 0.4);
  }

  .validation.invalid {
    background: rgba(239, 68, 68, 0.12);
    color: #7f1d1d;
    border: 1px solid rgba(239, 68, 68, 0.4);
  }

  .disabled-card {
    background: var(--surface);
    border: 1px solid var(--border);
    padding: 18px;
    border-radius: var(--radius);
  }

  .disabled-label {
    margin: 0;
    font-weight: 700;
    font-size: 1.05rem;
  }

  .muted {
    margin: 6px 0 0 0;
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  .connection-status {
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }

  .connection-status.failed {
    gap: 8px;
  }

  .loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border);
    border-top-color: #3b82f6;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .connection-text {
    margin: 0;
    font-size: 0.95rem;
    color: var(--text-muted);
  }

  .connection-text.error {
    color: #ef4444;
    font-weight: 600;
  }
  .loading {
    color: var(--muted);
    font-style: italic;
  }

  .status-message {
    margin: 0;
    color: var(--muted);
  }

  @media (max-width: 640px) {
    .tab-header {
      flex-direction: column;
      align-items: stretch;
    }

    .segmented {
      width: 100%;
    }

    .actions-row {
      justify-content: stretch;
    }

    .btn-primary {
      width: 100%;
      text-align: center;
    }
  }
</style>
