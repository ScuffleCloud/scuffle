<script lang="ts">
    import IconEventsError from '$lib/images/icon-events-error.svelte';
    import IconEventsNeutral from '$lib/images/icon-events-neutral.svelte';
    import IconEvents from '$lib/images/icon-events.svelte';
    import { EVENT_ICONS, type StreamEvent } from './types';

    const { events } = $props<{ events: StreamEvent[] }>();
    console.log(events);
</script>

<!-- TODO: Implement pagination and send actual events to this -->
<div class="events-container">
    <div class="events-list-header">
        <IconEvents />
        <h2>Events</h2>
    </div>
    <div class="events-list">
        {#each events as event, index}
            <!-- TODO: fix this keying. should come from the same object -->
            {@const Icon = EVENT_ICONS[event.type as keyof typeof EVENT_ICONS]}
            <div
                class="event-item"
                class:first={index === 0}
                class:last={index === events.length - 1}
            >
                <div class="event-content">
                    <span class="event-icon">
                        <Icon />
                    </span>
                    <span class="event-text">{event.text}</span>
                </div>
                <span class="event-time">{event.timestamp}</span>
            </div>
        {/each}
    </div>
    <div class="events-list-pagination">
        <button>Previous</button>
        <button>Next</button>
    </div>
</div>

<style>
    .events-container {
        background: #e6dedb;
        border-radius: 8px;
        padding: 0.5rem;
        gap: 0.5rem;
        display: flex;
        flex-direction: column;
        box-shadow:
            0px 1px 2px 0px #e6dedb,
            -1px 1px 0px 0px #e6dedb,
            1px 1px 0px 0px #e6dedb,
            1px -1px 0px 0px #e6dedb,
            -1px -1px 0px 0px #e6dedb;

        .events-list-header {
            display: flex;
            align-items: center;
            padding: 0.5rem;
            gap: 0.5rem;
            h2 {
                font-size: 1.25rem;
                font-weight: 600;
                color: #1a1a1a;
            }
        }

        .events-list {
            display: grid;
            grid-template-columns: 1fr;
            gap: 0.25rem;

            .event-item {
                display: grid;
                grid-template-columns: 1fr auto;
                align-items: center;
                padding: 0.375rem;
                background: #f1eae7;
                transition: background-color 0.2s;

                &.first {
                    border-top-left-radius: 8px;
                    border-top-right-radius: 8px;
                }

                &.last {
                    border-bottom-left-radius: 8px;
                    border-bottom-right-radius: 8px;
                }

                &:hover {
                    background: #f1f1f1;
                }

                .event-content {
                    display: grid;
                    grid-template-columns: auto 1fr;
                    align-items: center;
                    gap: 0.25rem;

                    .event-icon {
                        font-size: 1.25rem;
                        line-height: 1;
                    }

                    .event-text {
                        font-size: 13px;
                        font-weight: 600;
                        line-height: 24px;
                    }
                }

                .event-time {
                    color: #666666;
                    font-size: 0.875rem;
                    font-family: monospace;
                }
            }
        }

        .events-list-pagination {
            padding: 0.5rem;
        }
    }
</style>
