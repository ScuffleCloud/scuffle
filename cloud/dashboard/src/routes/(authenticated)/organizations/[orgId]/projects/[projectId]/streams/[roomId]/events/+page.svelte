<script lang="ts">
    import EventsTab from "$lib/components/streams/events/events-tab.svelte";
    import type { VideoStream } from "$lib/components/streams/types.js";
    import type { Streamed } from "$lib/types.js";

    // From parent layout
    type Props = {
        data: {
            stream: Streamed<VideoStream>;
        };
    };

    const { data }: Props = $props();
</script>

{#await data.stream}
    <div>Loading...</div>
{:then stream}
    <EventsTab events={stream.relatedStreams || []} />
{:catch error}
    <div>Error loading events: {error.message}</div>
{/await}
