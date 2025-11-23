<script lang="ts">
  import { onMount } from "svelte";
  import "$lib/themeStore";
  import "$lib/legacy-fallbacks.css";
  import "../app.css";

  onMount(() => {
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
  });

  // Fallback: Force hide loading screen after 3s if onMount fails to do it
  setTimeout(() => {
    const loading = document.getElementById("app-loading");
    if (loading && loading.style.display !== "none") {
      console.warn("DEBUG: Force hiding loading screen (fallback)");
      loading.style.display = "none";
    }
  }, 3000);
</script>

<slot />
