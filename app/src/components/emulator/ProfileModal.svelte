<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { onDestroy } from "svelte";
  import type { EmulatorProfile } from "$lib/api";
  import { pushError } from "$lib/notifications";
  import { formatErrorMessage } from "$lib/errorMessages";
  import { selectDirectory } from "$lib/api";

  export let profile: EmulatorProfile | null = null;
  export let isOpen = false;

  const dispatch = createEventDispatcher();

  let name = "";
  let emulatorId = "";
  let defaultSavePaths: string[] = [];
  let filePatterns: string[] = [];
  let newPath = "";
  let newPattern = "";

  // Initialize form when profile changes or modal opens
  $: if (isOpen) {
    if (profile) {
      name = profile.name;
      emulatorId = profile.emulator_id;
      defaultSavePaths = [...profile.default_save_paths];
      filePatterns = [...profile.file_patterns];
    } else {
      resetForm();
    }
    // Lock body scroll
    document.body.style.overflow = "hidden";
  } else {
    // Restore body scroll
    document.body.style.overflow = "";
  }

  onDestroy(() => {
    document.body.style.overflow = "";
  });

  function resetForm() {
    name = "";
    emulatorId = "";
    defaultSavePaths = [];
    filePatterns = [];
    newPath = "";
    newPattern = "";
  }

  function handleSave() {
    if (!name.trim()) {
      pushError("Profile name is required");
      return;
    }
    if (!emulatorId.trim()) {
      pushError("Emulator ID is required");
      return;
    }

    const updatedProfile: EmulatorProfile = {
      name: name.trim(),
      emulator_id: emulatorId.trim(),
      default_save_paths: defaultSavePaths,
      file_patterns: filePatterns,
    };

    dispatch("save", updatedProfile);
    close();
  }

  function close() {
    dispatch("close");
  }

  function addPath() {
    if (newPath.trim()) {
      defaultSavePaths = [...defaultSavePaths, newPath.trim()];
      newPath = "";
    }
  }

  function removePath(index: number) {
    defaultSavePaths = defaultSavePaths.filter((_, i) => i !== index);
  }

  function addPattern() {
    if (newPattern.trim()) {
      filePatterns = [...filePatterns, newPattern.trim()];
      newPattern = "";
    }
  }

  function removePattern(index: number) {
    filePatterns = filePatterns.filter((_, i) => i !== index);
  }

  async function browsePath() {
    try {
      const selectedPath = await selectDirectory();
      if (selectedPath) {
        defaultSavePaths = [...defaultSavePaths, selectedPath];
      }
    } catch (error) {
      pushError(formatErrorMessage(error));
    }
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal-backdrop" on:click={close}>
    <div
      class="modal-content settings-card"
      on:click|stopPropagation
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <header class="modal-header">
        <h3>{profile ? "Edit Profile" : "Add New Profile"}</h3>
        <button class="close-btn" on:click={close} aria-label="Close">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><line x1="18" y1="6" x2="6" y2="18" /><line
              x1="6"
              y1="6"
              x2="18"
              y2="18"
            /></svg
          >
        </button>
      </header>

      <div class="modal-body">
        <div class="input-stack">
          <div class="setting-item">
            <div class="form-group">
              <label for="name">Profile Name</label>
              <input
                type="text"
                id="name"
                bind:value={name}
                placeholder="e.g. Yuzu"
                class="input-field"
              />
            </div>
          </div>

          <div class="divider"></div>

          <div class="setting-item">
            <div class="form-group">
              <label for="emulatorId">Emulator ID</label>
              <input
                type="text"
                id="emulatorId"
                bind:value={emulatorId}
                placeholder="e.g. yuzu"
                class="input-field"
                disabled={!!profile}
              />
              {#if profile}
                <span class="hint">ID cannot be changed after creation.</span>
              {/if}
            </div>
          </div>
        </div>

        <div class="section-label">Configuration</div>

        <div class="input-stack">
          <div class="setting-item">
            <div class="form-group">
              <span class="label-text">Default Save Paths</span>
              <div class="list-input-row">
                <input
                  type="text"
                  bind:value={newPath}
                  placeholder="Add a path..."
                  class="input-field"
                  on:keydown={(e) => e.key === "Enter" && addPath()}
                />
                <button class="btn-secondary" on:click={browsePath}
                  >Browse...</button
                >
                <button class="btn-secondary" on:click={addPath}>Add</button>
              </div>
              {#if defaultSavePaths.length > 0}
                <ul class="item-list">
                  {#each defaultSavePaths as path, i}
                    <li>
                      <span class="item-text">{path}</span>
                      <button class="btn-icon" on:click={() => removePath(i)}
                        >&times;</button
                      >
                    </li>
                  {/each}
                </ul>
              {/if}
            </div>
          </div>

          <div class="divider"></div>

          <div class="setting-item">
            <div class="form-group">
              <span class="label-text">File Patterns (Glob)</span>
              <div class="list-input-row">
                <input
                  type="text"
                  bind:value={newPattern}
                  placeholder="e.g. *.sav"
                  class="input-field"
                  on:keydown={(e) => e.key === "Enter" && addPattern()}
                />
                <button class="btn-secondary" on:click={addPattern}>Add</button>
              </div>
              {#if filePatterns.length > 0}
                <ul class="item-list">
                  {#each filePatterns as pattern, i}
                    <li>
                      <span class="item-text">{pattern}</span>
                      <button class="btn-icon" on:click={() => removePattern(i)}
                        >&times;</button
                      >
                    </li>
                  {/each}
                </ul>
              {/if}
            </div>
          </div>
        </div>
      </div>

      <footer class="modal-footer">
        <button class="btn-text" on:click={close}>Cancel</button>
        <button class="btn-primary" on:click={handleSave}>Save Profile</button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    padding: 16px;
  }

  .modal-content {
    background: rgb(var(--surface-rgb));
    border: 1px solid var(--border);
    border-radius: var(--radius);
    width: 100%;
    max-width: 550px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: var(--shadow-lg);
    animation: slideUp 0.2s ease-out;
  }

  @media (max-width: 600px) {
    .modal-content {
      max-width: 100%;
      max-height: 95vh;
      border-radius: var(--radius-sm);
    }

    .modal-header,
    .modal-body,
    .modal-footer {
      padding: 16px;
    }

    .setting-item {
      padding: 12px;
    }

    .modal-body {
      padding: 20px 16px;
    }
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .modal-header {
    padding: 20px 24px;
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-radius: var(--radius) var(--radius) 0 0;
    background: rgb(var(--surface-rgb));
  }

  .modal-header h3 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text);
  }

  .close-btn {
    background: none;
    border: none;
    padding: 4px;
    cursor: pointer;
    color: var(--text-secondary);
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  .modal-body {
    padding: 24px;
    overflow-y: auto;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    gap: 24px;
    background: rgb(var(--surface-rgb));
    flex: 1 1 auto;
    min-height: 0;
  }

  .section-label {
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
    margin-bottom: -16px;
    margin-top: 8px;
    padding-left: 4px;
  }

  /* Input Stack Styling */
  .input-stack {
    display: grid;
    gap: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: rgb(var(--surface-muted-rgb));
  }

  .setting-item {
    padding: 16px;
    background: rgb(var(--surface-rgb));
  }

  .input-stack .setting-item:first-child {
    border-radius: var(--radius) var(--radius) 0 0;
  }

  .input-stack .setting-item:last-child {
    border-radius: 0 0 var(--radius) var(--radius);
  }

  .input-stack .setting-item:only-child {
    border-radius: var(--radius);
  }

  .divider {
    height: 1px;
    background: var(--border);
    margin: 0 16px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .form-group label,
  .label-text {
    font-weight: 500;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .input-field {
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 0.95rem;
    transition: all 0.2s;
  }

  .input-field:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 20%, transparent);
  }

  .input-field:disabled {
    opacity: 0.7;
    cursor: not-allowed;
    background: rgb(var(--surface-muted-rgb));
  }

  .hint {
    font-size: 0.8rem;
    color: var(--muted);
  }

  .list-input-row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .list-input-row .input-field {
    flex: 1 1 auto;
    min-width: 200px;
  }

  .list-input-row .btn-secondary {
    flex: 0 0 auto;
  }

  /* Mobile: input full width, buttons on second row */
  @media (max-width: 500px) {
    .list-input-row .input-field {
      flex: 1 1 100%;
      min-width: 100%;
    }

    .list-input-row .btn-secondary {
      flex: 1 1 auto;
      padding: 10px 16px;
    }
  }

  .item-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 8px;
  }

  .item-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: rgb(var(--surface-muted-rgb));
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    font-size: 0.9rem;
  }

  .item-text {
    font-family: "SF Mono", "Monaco", monospace;
    word-break: break-all;
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .btn-icon {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 1.2rem;
    padding: 0 4px;
    line-height: 1;
    opacity: 0.6;
    transition: opacity 0.2s;
  }

  .btn-icon:hover {
    color: var(--danger);
    opacity: 1;
  }

  .modal-footer {
    padding: 20px 24px;
    border-top: 1px solid var(--border);
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    background: rgb(var(--surface-muted-rgb));
    border-radius: 0 0 var(--radius) var(--radius);
  }

  .btn-text {
    background: none;
    border: none;
    padding: 8px 16px;
    cursor: pointer;
    color: var(--text-secondary);
    font-weight: 500;
    transition: color 0.2s;
  }

  .btn-text:hover {
    color: var(--text);
  }

  .btn-primary {
    background: var(--accent);
    color: white;
    border: none;
    padding: 8px 24px;
    border-radius: var(--radius-sm);
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: var(--shadow-sm);
  }

  .btn-primary:hover {
    opacity: 0.9;
    transform: translateY(-1px);
    box-shadow: var(--shadow);
  }

  .btn-secondary {
    background: rgb(var(--surface-rgb));
    border: 1px solid var(--border);
    color: var(--text);
    padding: 8px 16px;
    border-radius: var(--radius-sm);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    min-height: 40px;
    white-space: nowrap;
  }

  .btn-secondary:hover {
    background: var(--bg-hover);
    border-color: var(--text-secondary);
  }
</style>
