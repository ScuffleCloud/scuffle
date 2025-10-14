<script lang="ts">
    import AllVideoStreams from "$lib/components/streams/all-video-streams.svelte";
    import type { VideoStream } from "$lib/components/streams/types";
    import type { ListResponse, Streamed } from "$lib/types";

    type Props = {
        data: {
            streams: Streamed<ListResponse<VideoStream>>;
        };
    };

    const { data: pageData }: Props = $props();
</script>

<svelte:head>
    <title>Video | Live Streams | Scuffle</title>
</svelte:head>
<div class="page-bg">
    <AllVideoStreams streamedData={pageData.streams} />
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
