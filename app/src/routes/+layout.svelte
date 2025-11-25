<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import SyncConflictDialog from "../components/dialogs/SyncConflictDialog.svelte";
  import "$lib/themeStore";
  import "$lib/legacy-fallbacks.css";
  import "../app.css";

  let conflictDialogOpen = false;
  let conflictGameId = "";
  let conflictLocalTime = 0;
  let conflictCloudTime = 0;

  onMount(async () => {
    console.log("DEBUG: +layout.svelte mounted");
    const loading = document.getElementById("app-loading");
    if (loading) {
      console.log("DEBUG: Hiding loading screen");
      loading.style.opacity = "0";
      setTimeout(() => {
        loading.style.display = "none";
      }, 500);
    } else {
      console.error("DEBUG: Loading screen element not found!");
    }

    // Listen for sync conflicts
    await listen("sync://conflict-detected", async (event) => {
      const gameId = event.payload as string;
      console.log("[CONFLICT] Detected for game:", gameId);

      try {
        const details = (await invoke("get_conflict_details", {
          gameId,
        })) as { local_timestamp: number; cloud_timestamp: number };

        conflictGameId = gameId;
        conflictLocalTime = details.local_timestamp;
        conflictCloudTime = details.cloud_timestamp;
        conflictDialogOpen = true;
      } catch (error) {
        console.error("[CONFLICT] Failed to get details:", error);
      }
    });
  });

  async function handleConflictResolve(event: CustomEvent) {
    const { action } = event.detail;

    try {
      if (action === "upload") {
        await invoke("resolve_conflict_upload", { gameId: conflictGameId });
        console.log("[CONFLICT] Resolved: uploading local save");
      } else if (action === "download") {
        await invoke("resolve_conflict_download", { gameId: conflictGameId });
        console.log("[CONFLICT] Resolved: downloading cloud save");
      } else {
        console.log("[CONFLICT] Skipped for later");
      }
    } catch (error) {
      console.error("[CONFLICT] Resolution failed:", error);
    }
  }

  // Fallback: Force hide loading screen after 3s if onMount fails to do it
  setTimeout(() => {
    const loading = document.getElementById("app-loading");
    if (loading && loading.style.display !== "none") {
      console.warn("DEBUG: Force hiding loading screen (fallback)");
      loading.style.display = "none";
    }
  }, 3000);
</script>

<SyncConflictDialog
  bind:isOpen={conflictDialogOpen}
  gameId={conflictGameId}
  localTimestamp={conflictLocalTime}
  cloudTimestamp={conflictCloudTime}
  on:resolve={handleConflictResolve}
/>

<slot />
