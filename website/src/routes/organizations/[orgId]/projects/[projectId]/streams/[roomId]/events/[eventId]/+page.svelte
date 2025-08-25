<script lang="ts">
    import EventsTab from '$components/streams/events/events-tab.svelte';
    import { page } from '$app/state';
    import type { Streamed } from '$lib/types.js';
    import type { ChartData, VideoStream } from '$components/streams/types.js';

    type Props = {
        data: {
            // From parent
            stream: Streamed<VideoStream>;
            // From page load
            eventDetails: Streamed<ChartData>;
        };
    };

    const { data }: Props = $props();

    const currentEventId = $derived(page.params.eventId);

    const events = $derived.by(async () => {
        const stream = await data.stream;
        return stream.relatedStreams;
    });

    const { eventDetails } = data;
</script>

{#await Promise.all([events, eventDetails])}
    <div>Loading...</div>
{:then [resolvedEvents, resolvedEventDetails]}
    <EventsTab events={resolvedEvents || []} eventDetails={resolvedEventDetails} {currentEventId} />
{:catch error}
    <div>Error loading data: {error.message}</div>
{/await}
