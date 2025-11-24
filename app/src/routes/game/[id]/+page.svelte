<script lang="ts">
  import { page } from "$app/stores";
  import GameDetailPage from "../../../components/pages/GameDetailPage.svelte";

  $: gameId = $page.params.id;
</script>

<GameDetailPage {gameId} />
