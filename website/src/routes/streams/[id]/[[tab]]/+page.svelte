<script lang="ts">
    import type { VideoStream } from '$components/streams/types';
    import VideoStreamDetails from '$components/streams/VideoStreamDetails.svelte';
    import type { Streamed } from '$lib/types';

    type Props = {
        data: {
            stream: Streamed<VideoStream>;
        };
    };

    const { data: pageData }: Props = $props();
</script>

<svelte:head>
    <title>Video | Live Streams | Scuffle</title>
</svelte:head>
<div class="page-bg">
    <VideoStreamDetails streamedData={pageData.stream} />
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
