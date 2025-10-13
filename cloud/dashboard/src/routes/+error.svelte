<script lang="ts">
    import { page } from "$app/state";
</script>

<svelte:head>
    <title>Scuffle - Error</title>
</svelte:head>

<div class="container">
    <h3>{page.status}</h3>
    <p>{page.error?.message}</p>
    <a href="/">Go Back</a>
</div>

<style>
    .container {
      display: flex;
      flex-direction: column;
      justify-content: center;
      gap: 1rem;
      text-align: center;
      min-height: 60vh;
    }
</style>
