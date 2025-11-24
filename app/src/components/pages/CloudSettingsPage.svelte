<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import AppHeader from "../layout/AppHeader.svelte";
  import {
    cloudStore,
    type CloudAuthMode,
    type CloudConfig,
    type CloudMode,
    type CloudValidationResult,
  } from "$lib/stores/cloudStore";

  const modeStore = cloudStore.cloudMode;
  const configStore = cloudStore.cloudConfig;
  const validationStore = cloudStore.validation;
  const onlineStatusStore = cloudStore.onlineStatus;

  const modes: { label: string; value: CloudMode }[] = [
    { label: "Official Cloud", value: "official" },
    { label: "Self-host", value: "self_host" },
    { label: "Sync Off", value: "off" },
  ];

  const authOptions: { label: string; value: CloudAuthMode }[] = [
    { label: "NONE", value: "NONE" },
    { label: "ACCESS_KEY", value: "ACCESS_KEY" },
    { label: "USERPASS", value: "USERPASS" },
  ];

  let activeMode: CloudMode = "off";
  let onlineStatus: "online" | "offline" = "offline";
  let localConfig: CloudConfig = { mode: "off", auth_mode: "NONE" };
  let validation: CloudValidationResult = { status: "idle", message: "" };
  let loading = true;
  let saving = false;
  let validating = false;
  let statusMessage = "";

  $: activeMode = ($modeStore as CloudMode) ?? "off";
  $: onlineStatus = ($onlineStatusStore as "online" | "offline") ?? "offline";
  $: validation = ($validationStore as CloudValidationResult) ?? {
    status: "idle",
    message: "",
  };
  $: if ($configStore) {
    localConfig = { ...localConfig, ...$configStore };
  }

  onMount(async () => {
    await cloudStore.initialize();
    const config = (await cloudStore.getCloudConfig()) ?? {};
    localConfig = { ...localConfig, ...config };
    activeMode = (config.mode as CloudMode) ?? activeMode;
    loading = false;
  });

  function goBack() {
    goto("/settings", { keepFocus: true, noScroll: true });
  }

  async function handleModeChange(mode: CloudMode) {
    if (mode === activeMode) return;
    saving = true;
    statusMessage = "";
    try {
      await cloudStore.updateCloudMode(mode);
      await cloudStore.updateCloudSettings({ mode });
      if (mode === "off") {
        validation = { status: "idle", message: "" };
      }
    } catch (error) {
      statusMessage =
        typeof error === "string"
          ? error
          : ((error as Error)?.message ?? "Failed to update mode");
    } finally {
      saving = false;
    }
  }

  async function updateSelfHostField(
    field: keyof CloudConfig,
    value: string | CloudAuthMode
  ) {
    if (activeMode !== "self_host") return;
    saving = true;
    statusMessage = "";
    const updated = { ...localConfig, [field]: value, mode: "self_host" };
    localConfig = updated;
    try {
      await cloudStore.updateCloudSettings(updated);
      await cloudStore.validateSelfHostSettings(updated);
    } catch (error) {
      statusMessage =
        typeof error === "string"
          ? error
          : ((error as Error)?.message ?? "Failed to update settings");
    } finally {
      saving = false;
    }
  }

  async function updateOfficialField(field: keyof CloudConfig, value: string) {
    if (activeMode !== "official") return;
    saving = true;
    statusMessage = "";
    const updated = { ...localConfig, [field]: value, mode: "official" };
    localConfig = updated;
    try {
      await cloudStore.updateCloudSettings(updated);
    } catch (error) {
      statusMessage =
        typeof error === "string"
          ? error
          : ((error as Error)?.message ?? "Failed to update settings");
    } finally {
      saving = false;
    }
  }

  async function validateOfficial() {
    validating = true;
    statusMessage = "";
    try {
      await cloudStore.validateOfficialCloudSettings(localConfig);
    } catch (error) {
      statusMessage =
        typeof error === "string"
          ? error
          : ((error as Error)?.message ?? "Validation failed");
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
              <div class="segmented" role="tablist" aria-label="Cloud mode">
                {#each modes as mode}
                  <button
                    type="button"
                    role="tab"
                    aria-pressed={activeMode === mode.value}
                    class:active={activeMode === mode.value}
                    on:click={() => handleModeChange(mode.value)}
                    disabled={saving}
                  >
                    {mode.label}
                  </button>
                {/each}
              </div>

              <div class="status-chip" aria-live="polite">
                <span class={`dot ${onlineStatus}`}></span>
                <span class="status-text"
                  >{onlineStatus === "online" ? "Online" : "Offline"}</span
                >
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
              <div class="settings-card">
                <div class="card-content">
                  <div class="input-group">
                    <label for="official-base">Base URL</label>
                    <input
                      id="official-base"
                      type="url"
                      placeholder="https://cloud.crosssave.app"
                      value={localConfig.base_url ?? ""}
                      on:input={(event) =>
                        updateOfficialField(
                          "base_url",
                          (event.currentTarget as HTMLInputElement).value
                        )}
                    />
                  </div>

                  <div class="input-group">
                    <label for="official-api">API Key</label>
                    <input
                      id="official-api"
                      type="password"
                      placeholder="Enter API key"
                      value={localConfig.api_key ?? ""}
                      on:input={(event) =>
                        updateOfficialField(
                          "api_key",
                          (event.currentTarget as HTMLInputElement).value
                        )}
                    />
                  </div>

                  <div class="actions-row">
                    <button
                      class="btn-primary"
                      on:click={validateOfficial}
                      disabled={validating}
                    >
                      {validating ? "Validating..." : "Validate Settings"}
                    </button>
                  </div>

                  {#if validation.status !== "idle"}
                    <p class={`validation ${validation.status}`}>
                      {validation.message}
                    </p>
                  {/if}
                </div>
              </div>
            {:else if activeMode === "self_host"}
              <div class="settings-card">
                <div class="card-content">
                  <div class="input-group">
                    <label for="self-base">Base URL</label>
                    <input
                      id="self-base"
                      type="url"
                      placeholder="https://my-cloud.local"
                      value={localConfig.base_url ?? ""}
                      on:input={(event) =>
                        updateSelfHostField(
                          "base_url",
                          (event.currentTarget as HTMLInputElement).value
                        )}
                    />
                  </div>

                  <div class="input-group">
                    <label for="self-access">Access Key</label>
                    <input
                      id="self-access"
                      type="password"
                      placeholder="Access key"
                      value={localConfig.access_key ?? ""}
                      on:input={(event) =>
                        updateSelfHostField(
                          "access_key",
                          (event.currentTarget as HTMLInputElement).value
                        )}
                    />
                  </div>

                  <div class="input-group">
                    <label>Auth Mode</label>
                    <div
                      class="segmented auth"
                      role="group"
                      aria-label="Auth mode"
                    >
                      {#each authOptions as option}
                        <button
                          type="button"
                          class:active={localConfig.auth_mode === option.value}
                          aria-pressed={localConfig.auth_mode === option.value}
                          on:click={() =>
                            updateSelfHostField("auth_mode", option.value)}
                        >
                          {option.label}
                        </button>
                      {/each}
                    </div>
                  </div>

                  {#if validation.status !== "idle"}
                    <p class={`validation ${validation.status}`}>
                      {validation.message}
                    </p>
                  {/if}
                </div>
              </div>
            {/if}

            {#if statusMessage}
              <p class="status-message">{statusMessage}</p>
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
    background: #ef4444;
  }

  .settings-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--shadow-soft);
    overflow: hidden;
  }

  .card-content {
    padding: 16px;
    display: grid;
    gap: 16px;
  }

  .input-group {
    display: grid;
    gap: 8px;
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
    border: 1px dashed var(--border);
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
    color: var(--muted);
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
