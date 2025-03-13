<script lang="ts">
    import type { VideoStream } from '$components/streams/types';
    import VideoStreams from '$components/streams/VideoStreams.svelte';
    import type { Streamed } from '$lib/types';

    const { data: pageData } = $props<{
        data: {
            streams: Streamed<VideoStream[]>;
        };
    }>();
</script>

<div class="page-bg">
    <VideoStreams streamedData={pageData.streams} />
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
