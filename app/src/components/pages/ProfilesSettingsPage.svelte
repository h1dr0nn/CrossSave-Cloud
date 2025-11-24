<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import AppHeader from "../layout/AppHeader.svelte";
  import ProfileModal from "../emulator/ProfileModal.svelte";
  import ConfirmDialog from "../shared/ConfirmDialog.svelte";
  import {
    listProfiles,
    saveProfile,
    deleteProfile,
    validatePaths,
    type EmulatorProfile,
  } from "$lib/api";
  import { pushError, pushInfo, pushSuccess } from "$lib/notifications";

  function goBack() {
    goto("/settings", { keepFocus: true, noScroll: true });
  }

  let profiles: EmulatorProfile[] = [];
  let isModalOpen = false;
  let editingProfile: EmulatorProfile | null = null;
  let validation: Record<
    string,
    { status: string; message: string; validPaths: string[] } | undefined
  > = {};
  let deleteDialogOpen = false;
  let profileToDelete: EmulatorProfile | null = null;

  onMount(loadProfiles);

  async function loadProfiles() {
    try {
      profiles = await listProfiles();
    } catch (error) {
      pushError(`Failed to load profiles: ${error}`);
    }
  }

  function openAddModal() {
    editingProfile = null;
    isModalOpen = true;
  }

  function openEditModal(profile: EmulatorProfile) {
    editingProfile = profile;
    isModalOpen = true;
  }

  async function handleSaveProfile(event: CustomEvent<EmulatorProfile>) {
    try {
      await saveProfile(event.detail);
      await loadProfiles();
      pushSuccess(
        `Profile ${event.detail.name} ${editingProfile ? "updated" : "created"}`,
      );
      isModalOpen = false;
    } catch (error) {
      pushError(`Failed to save profile: ${error}`);
    }
  }

  async function handleDeleteProfile(profile: EmulatorProfile) {
    profileToDelete = profile;
    deleteDialogOpen = true;
  }

  async function confirmDelete() {
    if (!profileToDelete) return;

    try {
      await deleteProfile(profileToDelete.emulator_id);
      await loadProfiles();
      pushSuccess(`Profile ${profileToDelete.name} deleted`);
    } catch (error) {
      pushError(`Failed to delete profile: ${error}`);
    } finally {
      profileToDelete = null;
      deleteDialogOpen = false;
    }
  }

  async function handleValidate(profile: EmulatorProfile) {
    validation = {
      ...validation,
      [profile.emulator_id]: {
        status: "pending",
        message: "Validating...",
        validPaths: [],
      },
    };

    try {
      const validPaths = await validatePaths(profile.default_save_paths);
      validation = {
        ...validation,
        [profile.emulator_id]: {
          status: "ok",
          message: `${validPaths.length} path(s) validated`,
          validPaths,
        },
      };
      pushInfo(`Profile ${profile.emulator_id} validated`);

      // Auto-clear validation after 5 seconds
      setTimeout(() => {
        validation = {
          ...validation,
          [profile.emulator_id]: undefined,
        };
      }, 5000);
    } catch (error) {
      validation = {
        ...validation,
        [profile.emulator_id]: {
          status: "error",
          message: String(error),
          validPaths: [],
        },
      };
      pushError(`Validation failed for ${profile.emulator_id}: ${error}`);

      // Auto-clear error validation after 5 seconds
      setTimeout(() => {
        validation = {
          ...validation,
          [profile.emulator_id]: undefined,
        };
      }, 5000);
    }
  }
</script>

<section class="settings-page">
  <div class="content-surface">
    <main class="content-body">
      <div class="header-wrapper">
        <AppHeader
          title="Emulator Profiles"
          showBack
          onBack={goBack}
          onMenu={() => {}}
          sticky={false}
        />
      </div>

      <div class="settings-container">
        <div class="section-group">
          <div class="section-header">
            <p class="section-title">Manage Profiles</p>
            <button class="btn-primary-sm" on:click={openAddModal}>
              + Add Profile
            </button>
          </div>

          {#if profiles.length === 0}
            <div class="settings-card">
              <div class="card-content empty-state">
                <p>No emulator profiles found.</p>
                <button class="btn-secondary" on:click={openAddModal}
                  >Create your first profile</button
                >
              </div>
            </div>
          {:else}
            <div class="profiles-grid">
              {#each profiles as profile}
                <div class="settings-card profile-card">
                  <div class="card-header">
                    <div class="profile-info">
                      <h3>{profile.name}</h3>
                      <code class="profile-id">{profile.emulator_id}</code>
                    </div>
                    <div class="card-actions">
                      <button
                        class="icon-btn"
                        title="Edit"
                        on:click={() => openEditModal(profile)}
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="18"
                          height="18"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          ><path
                            d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"
                          ></path><path
                            d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"
                          ></path></svg
                        >
                      </button>
                      <button
                        class="icon-btn danger"
                        title="Delete"
                        on:click={() => handleDeleteProfile(profile)}
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="18"
                          height="18"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          ><polyline points="3 6 5 6 21 6"></polyline><path
                            d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
                          ></path><line x1="10" y1="11" x2="10" y2="17"
                          ></line><line x1="14" y1="11" x2="14" y2="17"
                          ></line></svg
                        >
                      </button>
                    </div>
                  </div>

                  <div class="card-content">
                    <div class="info-row">
                      <span class="label">Paths:</span>
                      <span class="value"
                        >{profile.default_save_paths.length} configured</span
                      >
                    </div>
                    <div class="info-row">
                      <span class="label">Patterns:</span>
                      <span class="value"
                        >{profile.file_patterns.join(", ") || "None"}</span
                      >
                    </div>

                    <div class="validation-section">
                      <button
                        class="btn-secondary-sm"
                        on:click={() => handleValidate(profile)}
                      >
                        Validate Paths
                      </button>
                      {#if validation[profile.emulator_id]}
                        <div
                          class={`validation-status ${validation[profile.emulator_id]?.status}`}
                        >
                          {validation[profile.emulator_id]?.message}
                        </div>
                      {/if}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </main>
  </div>
</section>

<ProfileModal
  isOpen={isModalOpen}
  profile={editingProfile}
  on:close={() => (isModalOpen = false)}
  on:save={handleSaveProfile}
/>

<ConfirmDialog
  isOpen={deleteDialogOpen}
  title="Delete Profile"
  message={profileToDelete
    ? `Are you sure you want to delete profile "${profileToDelete.name}"? This cannot be undone.`
    : ""}
  confirmLabel="Delete"
  cancelLabel="Cancel"
  isDanger={true}
  on:confirm={confirmDelete}
  on:cancel={() => {
    deleteDialogOpen = false;
    profileToDelete = null;
  }}
/>

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
    gap: 16px;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 4px;
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

  .profiles-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
  }

  .profile-card {
    display: flex;
    flex-direction: column;
  }

  .card-header {
    padding: 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    background: var(--bg-secondary);
  }

  .profile-info h3 {
    margin: 0 0 4px 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .profile-id {
    font-family: "SF Mono", "Monaco", monospace;
    font-size: 0.8rem;
    color: var(--text-secondary);
    background: var(--bg);
    padding: 2px 6px;
    border-radius: 4px;
    border: 1px solid var(--border);
  }

  .card-actions {
    display: flex;
    gap: 8px;
  }

  .icon-btn {
    background: none;
    border: none;
    padding: 4px;
    cursor: pointer;
    color: var(--text-secondary);
    border-radius: 4px;
    transition: all 0.2s;
  }

  .icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  .icon-btn.danger:hover {
    background: color-mix(in srgb, var(--danger) 10%, transparent);
    color: var(--danger);
  }

  .card-content {
    padding: 12px 16px;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.9rem;
    gap: 16px;
  }

  .info-row .label {
    color: var(--text-secondary);
    min-width: 80px;
    flex-shrink: 0;
  }

  .info-row .value {
    font-weight: 500;
    text-align: right;
    flex: 1;
  }

  .validation-section {
    margin-top: auto;
    padding-top: 12px;
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .validation-status {
    font-size: 0.85rem;
    padding: 6px 10px;
    border-radius: 4px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
  }

  .validation-status.ok {
    color: var(--success);
    background: color-mix(in srgb, var(--success) 10%, transparent);
    border-color: color-mix(in srgb, var(--success) 20%, transparent);
  }

  .validation-status.error {
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 10%, transparent);
    border-color: color-mix(in srgb, var(--danger) 20%, transparent);
  }

  .validation-status.pending {
    color: var(--warning);
    background: color-mix(in srgb, var(--warning) 10%, transparent);
    border-color: color-mix(in srgb, var(--warning) 20%, transparent);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px;
    gap: 16px;
    color: var(--text-secondary);
  }

  .btn-primary-sm {
    background: var(--accent);
    color: white;
    border: none;
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-weight: 600;
    font-size: 0.85rem;
    cursor: pointer;
    transition: opacity 0.2s;
  }

  .btn-primary-sm:hover {
    opacity: 0.9;
  }

  .btn-secondary-sm {
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-weight: 500;
    font-size: 0.85rem;
    cursor: pointer;
    width: 100%;
  }

  .btn-secondary-sm:hover {
    background: var(--bg-hover);
  }

  .btn-secondary {
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 8px 16px;
    border-radius: var(--radius-sm);
    font-weight: 500;
    cursor: pointer;
  }

  .btn-secondary:hover {
    background: var(--bg-hover);
  }
</style>
