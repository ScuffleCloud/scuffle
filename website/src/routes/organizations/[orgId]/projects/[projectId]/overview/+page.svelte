<script>
import Map from "$components/overview/world-map.svelte";
</script>

<svelte:head>
	<title>Overview | Scuffle</title>
</svelte:head>
<div class="page-bg">
	<Map mapData={null} />
</div>

<style>
.page-bg {
  background-color: var(--colors-light100);
  margin: 0 auto;
  width: 100%;
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem;
}
</style>
