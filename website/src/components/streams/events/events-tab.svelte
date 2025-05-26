<!-- This will hold all event stuff for a stream.
 we will have a dropdown selector that is defaulted to the current stream and can show past streams
 There will be an adjacent button to resume or stop live updates
 Then we will have our chart. We need to leverage a charting library to get this wroking properly probably but we will see
  Then there will be a

 -->

<script lang="ts">
    import StreamStatusPill from '$lib/shared-components/stream-status-pill.svelte';
    import EventsChart from './chart.svelte';
    import { Select } from 'melt/builders';
    import type { VideoStream } from '../types';
    import EventsList from './events-list.svelte';
    import type { StreamEvent } from './types';
    import EventsLegend from './events-legend.svelte';
    import IconPlayBig from '$lib/images/icon-play-big.svelte';

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

    const select = new Select<string>({
        value: streams[0].id,
    });

    const selectedStream = $derived(streams.find((stream) => stream.id === select.value));
</script>

<div class="events-tab-container">
    <div class="card">
        <div class="header">
            <button class="select-trigger" {...select.trigger}>
                <div class="stream-info">
                    {#if selectedStream}
                        <StreamStatusPill status={selectedStream.status} />
                        <span class="stream-id">{selectedStream.id}</span>
                    {/if}
                </div>
            </button>
            <div class="select-content" {...select.content}>
                <div class="select-header">
                    <span class="title">Current streams</span>
                    <div class="divider"></div>
                </div>
                {#each streams.filter((s) => s.status === 'live') as stream}
                    <div class="select-option" {...select.getOption(stream.id)}>
                        <StreamStatusPill status="live" />
                        <span class="stream-id">{stream.id}</span>
                    </div>
                {/each}
                <div class="select-header">
                    <span class="title">Past streams</span>
                    <div class="divider"></div>
                </div>
                {#each streams.filter((s) => s.status === 'finished') as stream}
                    <div class="select-option" {...select.getOption(stream.id)}>
                        <StreamStatusPill status="finished" />
                        <span class="stream-id">{stream.id}</span>
                    </div>
                {/each}
            </div>

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

        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1rem;
            padding: 1rem;
        }

        .select-trigger {
            display: inline-flex;
            align-items: center;
            padding: 0.5rem;
            background-color: white;
            border: 1px solid #e2e8f0;
            border-radius: 0.5rem;
            min-width: 400px;
            cursor: pointer;

            &:hover {
                background-color: #f8fafc;
            }
        }

        .select-content {
            background: white;
            border-radius: 0.375rem;
            border: 1px solid #e2e8f0;
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
            min-width: 400px;
            padding: 0.5rem;
            z-index: 50;

            /* Not sure why margins are being applied automatically to my select content */
            margin-top: 0rem;
            margin-left: 0rem;

            .select-header {
                padding: 0.5rem;
                color: #64748b;
                font-size: 0.875rem;
                font-weight: 500;
                display: flex;
                align-items: center;
                gap: 0.5rem;

                .title {
                    flex: 0 0 auto;
                }

                .divider {
                    width: 100%;
                    height: 1px;
                    background-color: #e2e8f0;
                }
            }

            .select-option {
                padding: 0.5rem;
                cursor: pointer;
                display: flex;
                align-items: center;
                gap: 0.5rem;

                &:hover {
                    background-color: #f1f5f9;
                }
            }
        }

        .stream-info {
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }

        .resume-button {
            padding: 0.5rem 1rem;
            background-color: #f1f5f9;
            border: none;
            border-radius: 0.375rem;
            font-size: 0.875rem;
            cursor: pointer;
            display: flex;
            align-items: center;
            border-radius: 0.5rem;
            background: var(--color-teal90);

            .resume-button-text {
                display: flex;
                padding: 0rem 0.25rem;
                justify-content: center;
                align-items: center;
                gap: 0.625rem;
                color: var(--color-brown90);
                font-size: 1rem;
                font-style: normal;
                font-weight: 700;
                line-height: 1.5rem; /* 150% */
            }

            &:hover {
                background-color: #e2e8f0;
            }
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
    }
</style>
