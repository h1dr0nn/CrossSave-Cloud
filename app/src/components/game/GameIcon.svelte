<script lang="ts">
  export let name: string;
  export let id: string;
  export let variant: string = "default";
  export let size: number = 48;
</script>

<svg
  viewBox="0 0 48 48"
  width={size}
  height={size}
  class="icon"
  data-variant={variant}
  aria-hidden="true"
>
  <defs>
    <linearGradient id="grad-{id}" x1="0" y1="0" x2="48" y2="48">
      {#if variant === "console"}
        <stop offset="0%" stop-color="#a78bfa" />
        <stop offset="100%" stop-color="#818cf8" />
      {:else if variant === "spark"}
        <stop offset="0%" stop-color="#fbbf24" />
        <stop offset="100%" stop-color="#f97316" />
      {:else}
        <stop offset="0%" stop-color="#c084fc" />
        <stop offset="100%" stop-color="#f472b6" />
      {/if}
    </linearGradient>
  </defs>
  <rect width="48" height="48" rx="12" fill="url(#grad-{id})" />
  <text
    x="24"
    y="24"
    text-anchor="middle"
    dominant-baseline="central"
    fill="white"
    font-size="20"
    font-weight="700"
    font-family="system-ui, -apple-system, sans-serif"
  >
    {name.charAt(0).toUpperCase()}
  </text>
</svg>

<style>
  .icon {
    display: block;
    border-radius: 16px;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.15);
  }
</style>
