<script lang="ts">
    import NewStream from '$components/streams/NewStream.svelte';
</script>

<svelte:head>
    <title>Video | Live Streams | Scuffle</title>
</svelte:head>
<div class="page-bg">
    <NewStream />
</div>

<style>
    .page-bg {
        background-color: var(--color-light100);
        margin: 0 auto;
        width: 100%;
        max-width: 1200px;
        margin: 0 auto;
        padding: 2rem;
    }
</style>
