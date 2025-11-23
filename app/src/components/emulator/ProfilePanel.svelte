<script lang="ts">
  import { onMount } from "svelte";
  import { listProfiles, validatePaths, type EmulatorProfile } from "$lib/api";
  import { pushError, pushInfo } from "$lib/notifications";

  interface ValidationState {
    status: "idle" | "pending" | "ok" | "error";
    message: string;
    validPaths: string[];
  }

  let profiles: EmulatorProfile[] = [];
  let validation: Record<string, ValidationState> = {};

  const loadProfiles = async () => {
    try {
      profiles = await listProfiles();
      pushInfo(`Loaded ${profiles.length} emulator profiles`);
    } catch (error) {
      pushError(`Failed to load profiles: ${error}`);
    }
  };

  const handleValidate = async (profile: EmulatorProfile) => {
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
    }
  };

  onMount(loadProfiles);
</script>

<section class="panel-content">
  <p class="description">
    List of emulator profiles loaded from resources. Validate default paths
    locally.
  </p>

  {#if profiles.length === 0}
    <p class="placeholder">No profiles loaded yet.</p>
  {:else}
    <div class="profiles">
      {#each profiles as profile}
        <div class="card">
          <header>
            <div>
              <p class="label">{profile.name}</p>
              <p class="id">{profile.emulator_id}</p>
            </div>
            <button on:click={() => handleValidate(profile)}>Validate</button>
          </header>

          <div class="section">
            <p class="section-title">Default Save Paths</p>
            <ul>
              {#each profile.default_save_paths as path}
                <li>{path}</li>
              {/each}
            </ul>
          </div>

          <div class="section">
            <p class="section-title">Glob Patterns</p>
            <ul>
              {#each profile.file_patterns as pattern}
                <li>{pattern}</li>
              {/each}
            </ul>
          </div>

          {#if validation[profile.emulator_id]}
            <div class={`status ${validation[profile.emulator_id].status}`}>
              <p>{validation[profile.emulator_id].message}</p>
              {#if validation[profile.emulator_id].validPaths.length}
                <ul>
                  {#each validation[profile.emulator_id].validPaths as path}
                    <li>{path}</li>
                  {/each}
                </ul>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</section>

<style>
  .panel-content {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .description {
    margin: 0;
    color: #475569;
    font-size: 14px;
  }

  .profiles {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .card {
    border: 1px solid color-mix(in srgb, var(--border) 90%, transparent);
    border-radius: 8px;
    padding: 12px;
    background: var(--surface);
    box-shadow: var(--shadow-soft);
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
  }

  .label {
    margin: 0;
    font-weight: 700;
  }

  .id {
    margin: 0;
    color: #475569;
    font-family: monospace;
    font-size: 13px;
  }

  button {
    padding: 6px 10px;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    background: #f8fafc;
    cursor: pointer;
  }

  button:hover {
    background: #e2e8f0;
  }

  .section {
    margin-top: 10px;
  }

  .section-title {
    margin: 0 0 6px 0;
    font-weight: 600;
    font-size: 14px;
  }

  ul {
    margin: 0;
    padding-left: 18px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .status {
    margin-top: 10px;
    padding: 8px;
    border-radius: 6px;
    background: #f8fafc;
    border: 1px solid #cbd5e1;
  }

  .status.ok {
    border-color: #86efac;
    background: #f0fdf4;
  }

  .status.error {
    border-color: #fca5a5;
    background: #fff1f2;
  }

  .status.pending {
    border-color: #fbbf24;
    background: #fffbeb;
  }

  .placeholder {
    color: #94a3b8;
    margin: 0;
  }
</style>
