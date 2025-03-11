<script lang="ts">
    import VideoStreams from '$components/streams/VideoStreams.svelte';
</script>

<svelte:head>
    <title>Scuffle - Video Streams</title>
</svelte:head>
<div class="page-bg">
    <VideoStreams />
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
