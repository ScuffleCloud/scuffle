<script lang="ts">
    import IconEventsHeader from '$lib/images/icon-events-header.svelte';
    import { EVENT_ICONS, type StreamEvent } from './types';

    const { events } = $props<{ events: StreamEvent[] }>();
</script>

<!-- TODO: Implement pagination and send actual events to this -->
<div class="events-container">
    <div class="events-list-header">
        <IconEventsHeader />
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
    <div class="events-list-pagination">1-10 items of 21 available</div>
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
                color: #1a1a1a;
                font-size: 1rem;
                font-weight: 700;
                line-height: 1.5rem;
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
                background: var(--colors-teal50);
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
                        display: flex;
                        align-items: center;
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
