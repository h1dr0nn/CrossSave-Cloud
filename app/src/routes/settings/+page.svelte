<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { openPath } from "@tauri-apps/plugin-opener";

  import AppHeader from "../../components/AppHeader.svelte";
  import ThemeSelector from "../../components/ThemeSelector.svelte";
  import type { EmulatorProfile } from "$lib/api";
  import { settingsStore, type OverrideState } from "$lib/settingsStore";
  import { pushError } from "$lib/notifications";

  interface ProfileFormState {
    name: string;
    emulatorId: string;
    savePath: string;
    patterns: string;
    useOverride: boolean;
    overridePath: string;
  }

  const defaultPatterns = "*.sav, *.srm, *.state";

  const createEmptyForm = (): ProfileFormState => ({
    name: "",
    emulatorId: "",
    savePath: "",
    patterns: defaultPatterns,
    useOverride: false,
    overridePath: "",
  });

  let profileForm: ProfileFormState = createEmptyForm();
  let retentionDraft = 10;
  let autoDeleteDraft = true;
  let lastSelectedId: string | null = null;

  function goBack() {
    goto("/", { keepFocus: true, noScroll: true });
  }

  onMount(() => {
    void settingsStore.load();
  });

  $: state = $settingsStore;
  $: {
    if (!state.selectedProfileId && state.profiles.length > 0) {
      settingsStore.setSelectedProfile(state.profiles[0].emulator_id);
    }
  }
  $: selectedProfile =
    state.profiles.find((p) => p.emulator_id === state.selectedProfileId) ??
    null;
  $: if (state.appSettings) {
    retentionDraft = state.appSettings.retention_limit;
    autoDeleteDraft = state.appSettings.auto_delete;
  }
  $: if (state.selectedProfileId !== lastSelectedId) {
    lastSelectedId = state.selectedProfileId;
    if (selectedProfile) {
      profileForm = toFormState(selectedProfile, state.overrides);
    } else {
      profileForm = createEmptyForm();
    }
  }

  function toFormState(
    profile: EmulatorProfile,
    overrides: Record<string, OverrideState>
  ): ProfileFormState {
    const override = overrides[profile.emulator_id];
    return {
      name: profile.name,
      emulatorId: profile.emulator_id,
      savePath: profile.default_save_paths[0] ?? "",
      patterns: profile.file_patterns.join(", "),
      useOverride: override?.enabled ?? false,
      overridePath: override?.path ?? "",
    };
  }

  function handleSelect(profile: EmulatorProfile) {
    settingsStore.setSelectedProfile(profile.emulator_id);
  }

  function handleAddProfile() {
    settingsStore.setSelectedProfile(null);
    lastSelectedId = null;
    profileForm = createEmptyForm();
  }

  function validateForm(): string | null {
    if (!profileForm.name.trim()) return "Name is required";
    if (!profileForm.emulatorId.trim()) return "Emulator ID is required";
    if (!profileForm.savePath.trim()) return "Save path is required";
    const patterns = profileForm.patterns
      .split(/[,\n]/)
      .map((p) => p.trim())
      .filter(Boolean);
    if (patterns.length === 0)
      return "Please provide at least one glob pattern";
    return null;
  }

  async function handleSaveProfile() {
    const error = validateForm();
    if (error) {
      pushError(error);
      return;
    }

    const patterns = profileForm.patterns
      .split(/[,\n]/)
      .map((p) => p.trim())
      .filter(Boolean);

    const payload: EmulatorProfile = {
      emulator_id: profileForm.emulatorId.trim(),
      name: profileForm.name.trim(),
      default_save_paths: [profileForm.savePath.trim()],
      file_patterns: patterns,
    };

    await settingsStore.saveProfile(payload);

    settingsStore.setOverride(payload.emulator_id, {
      enabled: profileForm.useOverride,
      path: profileForm.overridePath.trim(),
    });
  }

  async function handleDeleteProfile() {
    if (!profileForm.emulatorId) {
      profileForm = createEmptyForm();
      return;
    }
    await settingsStore.deleteProfile(profileForm.emulatorId);
  }

  function handleCancel() {
    if (selectedProfile) {
      profileForm = toFormState(selectedProfile, state.overrides);
      return;
    }
    profileForm = createEmptyForm();
  }

  function resolveSavePath(profile: EmulatorProfile) {
    const override = state.overrides[profile.emulator_id];
    if (override?.enabled && override.path.trim()) return override.path.trim();
    return profile.default_save_paths[0] ?? "";
  }

  function formatBytes(size: number) {
    if (!size) return "0 B";
    const units = ["B", "KB", "MB", "GB"];
    let value = size;
    let unit = 0;
    while (value >= 1024 && unit < units.length - 1) {
      value /= 1024;
      unit += 1;
    }
    return `${value.toFixed(value >= 10 ? 0 : 1)} ${units[unit]}`;
  }

  async function openStorageFolder() {
    if (!state.storageInfo) return;
    try {
      await openPath(state.storageInfo.history_path);
    } catch (error) {
      pushError(`Unable to open storage: ${error}`);
    }
  }

  async function applyRetention() {
    if (!state.appSettings) return;
    await settingsStore.applySettings({
      retention_limit: retentionDraft,
      auto_delete: autoDeleteDraft,
    });
  }
</script>

<section class="settings">
  <AppHeader
    title="Settings"
    showBack
    onBack={goBack}
    onMenu={() => {}}
    sticky={false}
  />

  <div class="accordion-stack">
    <details open class="accordion">
      <summary>
        <div>
          <p class="eyebrow">Profiles</p>
          <h2>Emulator Profiles</h2>
        </div>
        <span class="hint">Manage emulator save paths</span>
      </summary>
      <div class="accordion-body">
        <div class="profile-grid">
          <div class="profile-list">
            <div class="list-header">
              <p class="section-title">Loaded Profiles</p>
              <button class="ghost" on:click={handleAddProfile}
                >Add Profile</button
              >
            </div>
            {#if state.loadingProfiles}
              <p class="muted">Loading profiles...</p>
            {:else if state.profiles.length === 0}
              <p class="muted">No profiles found.</p>
            {:else}
              <div class="cards">
                {#each state.profiles as profile}
                  <button
                    class={`card ${state.selectedProfileId === profile.emulator_id ? "active" : ""}`}
                    on:click={() => handleSelect(profile)}
                  >
                    <div>
                      <p class="label">{profile.name}</p>
                      <p class="id">{profile.emulator_id}</p>
                    </div>
                    <p class="path" title={resolveSavePath(profile)}>
                      {resolveSavePath(profile)}
                    </p>
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          <div class="profile-form">
            <div class="form-header">
              <div>
                <p class="section-title">
                  {selectedProfile ? "Edit Profile" : "New Profile"}
                </p>
                <p class="muted">Configure emulator metadata and save paths.</p>
              </div>
              <div class="actions">
                <button class="ghost" on:click={handleCancel}>Cancel</button>
                <button
                  class="secondary"
                  on:click={handleDeleteProfile}
                  disabled={state.savingProfile || !profileForm.emulatorId}
                >
                  Delete
                </button>
                <button
                  class="primary"
                  on:click={handleSaveProfile}
                  disabled={state.savingProfile}
                >
                  {state.savingProfile ? "Saving..." : "Save"}
                </button>
              </div>
            </div>

            <div class="field-group">
              <label for="name">Name</label>
              <input
                id="name"
                placeholder="RetroArch"
                bind:value={profileForm.name}
                autocomplete="off"
              />
            </div>
            <div class="field-group">
              <label for="emulatorId">Emulator ID</label>
              <input
                id="emulatorId"
                placeholder="retroarch"
                bind:value={profileForm.emulatorId}
                autocomplete="off"
              />
            </div>
            <div class="field-group">
              <label for="savePath">Save path</label>
              <input
                id="savePath"
                placeholder="~/saves/retroarch"
                bind:value={profileForm.savePath}
              />
              <p class="muted">
                Used as the default path unless override is enabled.
              </p>
            </div>
            <div class="field-group">
              <label for="patterns">Glob patterns</label>
              <input
                id="patterns"
                placeholder="*.sav, *.srm, *.state"
                bind:value={profileForm.patterns}
              />
              <p class="muted">Comma or newline separated patterns.</p>
            </div>

            <div class="override-row">
              <label class="toggle">
                <input type="checkbox" bind:checked={profileForm.useOverride} />
                <span>Override save path</span>
              </label>
              <input
                placeholder="/custom/path/to/saves"
                bind:value={profileForm.overridePath}
                class="override-input"
                aria-label="Override path"
                disabled={!profileForm.useOverride}
              />
            </div>
          </div>
        </div>
      </div>
    </details>

    <details open class="accordion">
      <summary>
        <div>
          <p class="eyebrow">Backups</p>
          <h2>Backup Retention</h2>
        </div>
        <span class="hint">Keep your versions tidy</span>
      </summary>
      <div class="accordion-body retention">
        <div class="field-group inline">
          <label for="retentionRange">Versions to keep</label>
          <div class="range-row">
            <input
              type="range"
              min={state.retentionBounds[0]}
              max={state.retentionBounds[1]}
              id="retentionRange"
              bind:value={retentionDraft}
            />
            <span class="value">{retentionDraft} versions</span>
          </div>
        </div>
        <label class="toggle">
          <input type="checkbox" bind:checked={autoDeleteDraft} />
          <span>Auto delete old versions</span>
        </label>
        <div class="actions">
          <button
            class="primary"
            on:click={applyRetention}
            disabled={state.savingSettings}
          >
            {state.savingSettings ? "Saving..." : "Save changes"}
          </button>
        </div>
      </div>
    </details>

    <details open class="accordion">
      <summary>
        <div>
          <p class="eyebrow">Storage</p>
          <h2>App Storage</h2>
        </div>
        <span class="hint">History cache and folders</span>
      </summary>
      <div class="accordion-body storage">
        <div class="stat">
          <p class="label">History folder</p>
          <p class="value">{state.storageInfo?.history_path ?? "-"}</p>
        </div>
        <div class="stat">
          <p class="label">Current size</p>
          <p class="value">
            {state.loadingStorage
              ? "Calculating..."
              : formatBytes(state.storageInfo?.size_bytes ?? 0)}
          </p>
        </div>
        <div class="actions">
          <button
            class="secondary"
            on:click={openStorageFolder}
            disabled={!state.storageInfo}
          >
            Open storage folder
          </button>
          <button class="primary" on:click={() => settingsStore.clearHistory()}>
            Clear all history cache
          </button>
        </div>
      </div>
    </details>

    <details open class="accordion">
      <summary>
        <div>
          <p class="eyebrow">Appearance</p>
          <h2>Theme</h2>
        </div>
        <span class="hint">Dark or light</span>
      </summary>
      <div class="accordion-body">
        <ThemeSelector />
      </div>
    </details>
  </div>
</section>

<style>
  .settings {
    padding: clamp(16px, 3vw, 32px);
    padding-top: max(clamp(16px, 3vw, 32px), env(safe-area-inset-top));
    padding-bottom: max(clamp(16px, 3vw, 32px), env(safe-area-inset-bottom));
    padding-left: max(clamp(16px, 3vw, 32px), env(safe-area-inset-left));
    padding-right: max(clamp(16px, 3vw, 32px), env(safe-area-inset-right));
    display: grid;
    grid-template-rows: auto 1fr;
    gap: 18px;
    background: var(--bg);
    color: var(--text);
    min-height: 100vh;
  }

  .accordion-stack {
    display: grid;
    gap: 14px;
    align-content: start;
  }

  .accordion {
    border: 1px solid color-mix(in srgb, var(--border) 85%, transparent);
    border-radius: var(--radius);
    background: var(--surface);
    box-shadow: var(--shadow-soft);
    overflow: hidden;
  }

  summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 14px 16px;
    cursor: pointer;
    user-select: none;
    background: color-mix(in srgb, var(--surface-muted) 60%, transparent);
  }

  summary::-webkit-details-marker {
    display: none;
  }

  .accordion-body {
    padding: 16px;
    background: var(--surface);
    display: grid;
    gap: 12px;
  }

  h2 {
    margin: 0;
    font-size: 1.2rem;
  }

  .hint {
    color: var(--muted);
    font-size: 0.95rem;
    white-space: nowrap;
  }

  .profile-grid {
    display: grid;
    grid-template-columns: minmax(240px, 320px) 1fr;
    gap: 16px;
    align-items: start;
  }

  .profile-list {
    display: grid;
    gap: 10px;
  }

  .list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .profile-form {
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    border-radius: var(--radius);
    padding: 14px;
    background: color-mix(in srgb, var(--surface-muted) 30%, transparent);
    display: grid;
    gap: 12px;
  }

  .cards {
    display: grid;
    gap: 10px;
  }

  .card {
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    border-radius: 12px;
    padding: 10px 12px;
    background: color-mix(in srgb, var(--surface-muted) 20%, transparent);
    text-align: left;
    display: grid;
    gap: 6px;
    cursor: pointer;
  }

  .card.active {
    border-color: color-mix(in srgb, var(--accent) 40%, var(--border));
    box-shadow: 0 8px 18px
      color-mix(in srgb, var(--accent-muted) 35%, transparent);
  }

  .label {
    margin: 0;
    font-weight: 700;
  }

  .id {
    margin: 0;
    color: var(--muted);
    font-size: 0.95rem;
  }

  .path {
    margin: 0;
    color: var(--text);
    font-size: 0.95rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .section-title {
    margin: 0;
    font-weight: 700;
  }

  .muted {
    color: var(--muted);
    margin: 0;
  }

  .field-group {
    display: grid;
    gap: 6px;
  }

  label {
    font-weight: 600;
  }

  input {
    width: 100%;
    padding: 10px;
    border-radius: 10px;
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    background: color-mix(in srgb, var(--surface) 90%, transparent);
    color: var(--text);
  }

  input:disabled {
    opacity: 0.6;
  }

  .override-row {
    display: grid;
    gap: 10px;
  }

  .override-input {
    width: 100%;
  }

  .toggle {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .toggle input {
    width: auto;
  }

  .form-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }

  .actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }

  button {
    border-radius: 10px;
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    padding: 10px 14px;
    cursor: pointer;
    background: color-mix(in srgb, var(--surface) 90%, transparent);
    color: var(--text);
  }

  button.primary {
    background: var(--accent);
    border-color: color-mix(in srgb, var(--accent) 60%, var(--border));
    color: #0b1222;
    font-weight: 700;
  }

  button.secondary {
    background: color-mix(in srgb, var(--surface-muted) 30%, transparent);
  }

  button.ghost {
    background: transparent;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .range-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .retention .value {
    min-width: 110px;
    text-align: right;
  }

  .stat {
    display: grid;
    gap: 2px;
  }

  .stat .label {
    color: var(--muted);
    font-weight: 600;
  }

  .stat .value {
    margin: 0;
    word-break: break-all;
  }

  .storage .actions {
    justify-content: flex-start;
  }

  .eyebrow {
    margin: 0;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    font-size: 0.8rem;
    color: var(--muted);
  }

  @media (max-width: 900px) {
    .profile-grid {
      grid-template-columns: 1fr;
    }

    summary {
      flex-direction: column;
      align-items: flex-start;
    }

    .hint {
      white-space: normal;
    }
  }
</style>
