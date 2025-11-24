<script lang="ts">
  import { onMount } from "svelte";
  import { cloudStore } from "$lib/stores/cloudStore";
  import { pushError, pushSuccess } from "$lib/notifications";

  let cloudConfig: any = null;
  let loading = true;
  let saving = false;
  let showApiKey = false;
  let deviceId = "";
  let formError = "";

  onMount(async () => {
    try {
      cloudConfig = await cloudStore.getCloudConfig();
      const status = await cloudStore.getCloudStatus();
      deviceId = status.device_id;
    } catch (error) {
      console.error("Failed to load cloud config:", error);
    } finally {
      loading = false;
    }
  });

  async function saveConfig() {
    formError = "";

    if (cloudConfig?.enabled) {
      if (!cloudConfig.base_url || !cloudConfig.base_url.trim()) {
        formError = "Base URL is required";
        pushError(formError);
        return;
      }
      if (!cloudConfig.api_key || !cloudConfig.api_key.trim()) {
        formError = "API key is required";
        pushError(formError);
        return;
      }
    }

    saving = true;
    try {
      await cloudStore.updateCloudConfig(cloudConfig);
      pushSuccess("Cloud settings saved successfully");
    } catch (error) {
      console.error("Failed to save cloud config:", error);
      formError =
        typeof error === "string"
          ? error
          : "Failed to save cloud settings";
      pushError(formError);
    } finally {
      saving = false;
    }
  }

  async function generateNewDeviceId() {
    if (
      confirm(
        "Generate a new Device ID? This will unlink this device from cloud sync."
      )
    ) {
      // TODO: Implement device ID regeneration in backend
      alert("Device ID regeneration not yet implemented");
    }
  }
</script>

<div class="cloud-settings">
  <h3>Cloud Sync</h3>

  {#if loading}
    <p class="loading">Loading cloud settings...</p>
  {:else if cloudConfig}
    <div class="setting-group">
      <label class="toggle-label">
        <input
          type="checkbox"
          bind:checked={cloudConfig.enabled}
          on:change={saveConfig}
        />
        <span>Enable Cloud Sync</span>
      </label>
      <p class="hint">Automatically sync your saves to the cloud</p>
    </div>

    <div class="setting-group">
      <label>
        <span>Base URL</span>
        <input
          type="url"
          bind:value={cloudConfig.base_url}
          placeholder="https://api.crosssave.local"
          disabled={!cloudConfig.enabled}
        />
      </label>
      <p class="hint">Cloud API endpoint</p>
    </div>

    <div class="setting-group">
      <label>
        <span>API Key</span>
        <div class="api-key-input">
          <input
            type={showApiKey ? "text" : "password"}
            bind:value={cloudConfig.api_key}
            placeholder="Enter your API key"
            disabled={!cloudConfig.enabled}
          />
          <button
            type="button"
            class="toggle-visibility"
            on:click={() => (showApiKey = !showApiKey)}
          >
            {showApiKey ? "🙈" : "👁️"}
          </button>
        </div>
      </label>
      <p class="hint">Your cloud authentication key</p>
    </div>

    <div class="setting-group">
      <label>
        <span>Device ID</span>
        <div class="device-id-display">
          <code>{deviceId}</code>
          <button
            type="button"
            class="btn-secondary"
            on:click={generateNewDeviceId}
          >
            Generate New
          </button>
        </div>
      </label>
      <p class="hint">Unique identifier for this device</p>
    </div>

    <div class="setting-group">
      <label>
        <span>Timeout (seconds)</span>
        <input
          type="number"
          bind:value={cloudConfig.timeout_seconds}
          min="5"
          max="300"
          disabled={!cloudConfig.enabled}
        />
      </label>
      <p class="hint">Request timeout duration</p>
    </div>

    {#if formError}
      <p class="error">{formError}</p>
    {/if}

    <button
      class="btn-primary save-btn"
      on:click={saveConfig}
      disabled={saving || !cloudConfig.enabled}
    >
      {saving ? "Saving..." : "Save Cloud Settings"}
    </button>
  {/if}
</div>

<style>
  .cloud-settings {
    padding: 1rem 0;
  }

  h3 {
    margin: 0 0 1.5rem 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .loading {
    color: var(--text-secondary);
    font-style: italic;
  }

  .setting-group {
    margin-bottom: 1.5rem;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
  }

  label span {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    cursor: pointer;
  }

  .toggle-label input[type="checkbox"] {
    width: 20px;
    height: 20px;
    cursor: pointer;
  }

  input[type="url"],
  input[type="password"],
  input[type="text"],
  input[type="number"] {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 0.95rem;
  }

  input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .api-key-input {
    display: flex;
    gap: 0.5rem;
  }

  .api-key-input input {
    flex: 1;
  }

  .toggle-visibility {
    padding: 0.75rem 1rem;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    background: var(--bg-secondary);
    cursor: pointer;
    font-size: 1.2rem;
  }

  .toggle-visibility:hover {
    background: var(--bg-hover);
  }

  .device-id-display {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .device-id-display code {
    flex: 1;
    padding: 0.75rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    font-family: "SF Mono", "Monaco", "Courier New", monospace;
    font-size: 0.85rem;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .hint {
    margin: 0.5rem 0 0 0;
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .btn-primary,
  .btn-secondary {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 8px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-primary {
    background: var(--accent-color);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .btn-secondary {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
  }

  .btn-secondary:hover {
    background: var(--bg-hover);
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .save-btn {
    margin-top: 1rem;
  }

  .error {
    color: var(--danger);
    margin: 0.5rem 0;
  }
</style>
