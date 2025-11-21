<script lang="ts">
  import GameDetailView from "../../../components/GameDetailView.svelte";

  export let params: { id: string };
</script>

<GameDetailView gameId={params.id} />
