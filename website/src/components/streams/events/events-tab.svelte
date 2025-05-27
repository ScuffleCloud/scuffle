<!-- This will hold all event stuff for a stream.
 we will have a dropdown selector that is defaulted to the current stream and can show past streams
 There will be an adjacent button to resume or stop live updates
 Then we will have our chart. We need to leverage a charting library to get this working properly probably but we will see
  Then there will be a

 -->

<script lang="ts">
    import EventsChart from './chart.svelte';
    import type { VideoStream } from '../types';
    import EventsList from './events-list.svelte';
    import type { StreamEvent } from './types';
    import EventsLegend from './events-legend.svelte';
    import IconPlayBig from '$lib/images/icon-play-big.svelte';
    import StreamSelect from './stream-select.svelte';

    const streams: VideoStream[] = [
        {
            id: '8a28e499d6s7987fd9812937fd981293',
            status: 'live',
            created: 'May 5, 04:01:11',
            name: 'Stream 1',
        },
        {
            id: '3223499d6s7987fd9812123123132123',
            status: 'finished',
            created: 'May 5, 04:01:11',
            name: 'Stream 2',
        },
        {
            id: '21ae213132s7987fd981212312312',
            status: 'finished',
            created: 'May 5, 04:01:11',
            name: 'Stream 3',
        },
        {
            id: '3e3120f5c4e4s7987fd981123123312',
            status: 'finished',
            created: 'May 5, 04:01:11',
            name: 'Stream 4',
        },
    ];

    // It doesn't matter what we fetch, we need to reformat the events before dumping it in our chart

    const events: StreamEvent[] = [
        {
            id: '1',
            type: 'info',
            text: 'Neutral event',
            timestamp: 'May 5, 04:01:11 PM EDT',
        },
        {
            id: '2',
            type: 'asset_created',
            text: 'Success event',
            timestamp: 'May 5, 04:01:11 PM EDT',
        },
        {
            id: '3',
            type: 'error',
            text: 'Error event',
            timestamp: 'May 5, 04:01:11 PM EDT',
        },
    ];

    let selectedStream = $state('');

    function handleStreamChange(value: string | undefined) {
        selectedStream = value || '';
    }
</script>

<div class="events-tab-container">
    <div class="card">
        <div class="header">
            <!-- TODO: Can use design system select here and migrate things when needed-->
            <StreamSelect
                {streams}
                bind:value={selectedStream}
                onValueChange={handleStreamChange}
            />
            <button class="resume-button">
                <div class="resume-button-text">Resume Live Updates</div>
                <IconPlayBig />
            </button>
        </div>

        <!-- For the data-zoom slider + chart -->
        <div class="events-chart-container">
            <EventsChart />
        </div>
        <div class="events-legend-container">
            <EventsLegend />
        </div>
    </div>

    <EventsList {events} />
</div>

<style>
    .events-tab-container {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .card {
        background: var(--color-teal30);
        border-radius: 8px;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;
    }

    .resume-button {
        padding: 0.5rem 1rem;
        background: var(--color-teal90);
        border: none;
        border-radius: 0.5rem;
        font-size: 0.875rem;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 0.625rem;
        transition: background-color 0.2s;
        flex-shrink: 0;
    }

    .resume-button-text {
        color: var(--color-brown90);
        font-size: 1rem;
        font-weight: 700;
        line-height: 1.5rem;
    }

    .resume-button:hover {
        background-color: #e2e8f0;
    }

    .events-chart-container {
        height: 250px;
        border-radius: 4px;
        width: 100%;
        padding: 0.25rem;
    }

    .events-legend-container {
        padding: 1.5rem 1rem 1rem 1rem;
        border-radius: 0rem 0rem 0.5rem 0.5rem;
        background: var(--color-teal70);
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }
</style>
