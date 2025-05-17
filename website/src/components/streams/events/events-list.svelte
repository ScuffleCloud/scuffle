<script lang="ts">
    import type { StreamEvent } from './types';

    const { events } = $props<{ events: StreamEvent[] }>();

    const getEventIcon = (type: StreamEvent['type']) => {
        switch (type) {
            case 'error':
                return '◆'; // diamond
            case 'info':
                return '■'; // square
            case 'success':
            case 'neutral':
            default:
                return '●'; // circle
        }
    };
</script>

<div class="events-container">
    <h2>Events</h2>
    <div class="events-list">
        {#each events as event}
            <div class="event-item">
                <div class="event-content">
                    <span
                        class="event-icon"
                        class:success={event.type === 'success' || event.type === 'neutral'}
                        class:error={event.type === 'error'}
                        class:info={event.type === 'info'}
                    >
                        {getEventIcon(event.type)}
                    </span>
                    <span class="event-text">{event.text}</span>
                </div>
                <span class="event-time">{event.timestamp}</span>
            </div>
        {/each}
    </div>
</div>

<style>
    .events-container {
        background: white;
        border-radius: 8px;
        padding: 1.5rem;

        h2 {
            font-size: 1.25rem;
            font-weight: 600;
            margin-bottom: 1rem;
            color: #1a1a1a;
        }

        .events-list {
            display: flex;
            flex-direction: column;
            gap: 0.25rem;

            .event-item {
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 1rem;
                background: #f8f8f8;
                border-radius: 4px;
                transition: background-color 0.2s;

                &:hover {
                    background: #f1f1f1;
                }

                .event-content {
                    display: flex;
                    align-items: center;
                    gap: 0.75rem;

                    .event-icon {
                        font-size: 1.25rem;
                        line-height: 1;

                        &.success {
                            color: #22c55e;
                        }

                        &.error {
                            color: #ef4444;
                        }

                        &.info {
                            color: #3b82f6;
                        }
                    }

                    .event-text {
                        font-family: monospace;
                        color: #1a1a1a;
                    }
                }

                .event-time {
                    color: #666666;
                    font-size: 0.875rem;
                    font-family: monospace;
                }
            }
        }
    }
</style>
