<script lang="ts">
  import { onMount } from "svelte";

  import MainLayout from "./components/MainLayout.svelte";

  let hasError = false;

  function handleError(event: ErrorEvent) {
    console.error("Uncaught error", event.error ?? event.message);
    hasError = true;
  }

  function handleUnhandledRejection(event: PromiseRejectionEvent) {
    console.error("Unhandled promise rejection", event.reason);
    hasError = true;
  }

  onMount(() => {
    window.addEventListener("error", handleError);
    window.addEventListener("unhandledrejection", handleUnhandledRejection);

    return () => {
      window.removeEventListener("error", handleError);
      window.removeEventListener("unhandledrejection", handleUnhandledRejection);
    };
  });
</script>

{#if !hasError}
  <MainLayout />
{:else}
  <div class="error-screen">
    <h1>Something went wrong</h1>
    <p>Check the developer console for more details.</p>
  </div>
{/if}

<style>
  .error-screen {
    min-height: 100vh;
    display: grid;
    place-items: center;
    text-align: center;
    color: var(--text);
    background: var(--bg);
    padding: 32px;
  }

  .error-screen h1 {
    margin-bottom: 8px;
  }

  .error-screen p {
    margin: 0;
    color: var(--muted);
  }
</style>
