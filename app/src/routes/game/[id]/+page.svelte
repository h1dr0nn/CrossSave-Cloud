<script lang="ts">
  import { page } from "$app/stores";
  import GameDetailView from "../../../components/game/GameDetailView.svelte";
</script>

<GameDetailView gameId={$page.params.id ?? ""} />
