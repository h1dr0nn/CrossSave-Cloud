<script lang="ts">
  import GameDetailView from "../game/GameDetailView.svelte";

  export let gameId: string;

  $: id = gameId;
</script>

<GameDetailView {gameId} />
