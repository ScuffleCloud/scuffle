<!-- This will hold all event stuff for a stream.
 we will have a dropdown selector that is defaulted to the current stream and can show past streams
 There will be an adjacent button to resume or stop live updates
 Then we will have our chart. We need to leverage a charting library to get this wroking properly probably but we will see
  Then there will be a

 -->

<script lang="ts">
    import StreamStatusPill from '$lib/shared-components/stream-status-pill.svelte';
    import EventsChart from './events-chart.svelte';
    import { Select } from 'melt/builders';
    import type { VideoStream } from '../types';

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

    const select = new Select<string>({
        value: streams[0].id,
    });

    $: selectedStream = streams.find((stream) => stream.id === select.value);
</script>

<div class="card">
    <div class="header">
        <div class="controls">
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
                    <span>Current streams</span>
                </div>
                {#each streams.filter((s) => s.status === 'live') as stream}
                    <div class="select-option" {...select.getOption(stream.id)}>
                        <span class="status-badge live">Live</span>
                        <span class="stream-id">{stream.id}</span>
                        <span class="stream-time">{stream.created}</span>
                    </div>
                {/each}

                <div class="select-header">
                    <span>Past streams</span>
                </div>
                {#each streams.filter((s) => s.status === 'finished') as stream}
                    <div class="select-option" {...select.getOption(stream.id)}>
                        <span class="status-badge">Finished</span>
                        <span class="stream-id">{stream.id}</span>
                        <span class="stream-time">{stream.created}</span>
                    </div>
                {/each}
            </div>

            <button class="resume-button"> Resume Live Updates ▶ </button>
        </div>
    </div>

    <div class="events-chart-container">
        <EventsChart />
    </div>

    <!-- <div class="events-list">
        <h3>Events</h3>
        <div class="event-items">
            <div class="event-item">
                <span class="event-icon">●</span>
                <span class="event-text">Neutral event</span>
                <span class="event-time">May 5, 04:01:11 PM EDT</span>
            </div>
        </div>
    </div> -->
</div>

<style>
    .card {
        background: white;
        border-radius: 8px;
        padding: 1rem;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);

        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1rem;

            .controls {
                display: flex;
                gap: 0.75rem;
                align-items: center;
            }
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

            .select-header {
                padding: 0.5rem;
                color: #64748b;
                font-size: 0.875rem;
                font-weight: 500;
                background-color: #f8fafc;
                border-bottom: 1px solid #e2e8f0;
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

        .status-badge {
            padding: 0.25rem 0.5rem;
            background: #e2e8f0;
            border-radius: 999px;
            font-size: 0.75rem;
            font-weight: 500;

            &.live {
                background: #ef4444;
                color: white;
            }
        }

        .stream-time {
            color: #64748b;
            font-size: 0.875rem;
        }

        .resume-button {
            padding: 0.5rem 1rem;
            background-color: #f1f5f9;
            border: none;
            border-radius: 0.375rem;
            font-size: 0.875rem;
            cursor: pointer;

            &:hover {
                background-color: #e2e8f0;
            }
        }

        .events-chart-container {
            height: 250px;
            margin-bottom: 1rem;
            border: 1px solid #eee;
            border-radius: 4px;
        }

        .event-items {
            display: flex;
            flex-direction: column;
            gap: 0.5rem;
        }

        .event-item {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.5rem;
            background: #f9f9f9;
            border-radius: 4px;

            .event-icon {
                color: #22c55e;
            }

            .event-time {
                color: #64748b;
                font-size: 0.875rem;
            }
        }
    }
</style>
