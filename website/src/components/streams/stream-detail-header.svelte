<!-- website/src/components/streams/StreamDetailsHeader.svelte -->
<script lang="ts">
    import IconDate from '$lib/images/icon-date.svelte';
    import StreamStatusPill from '$lib/shared-components/stream-status-pill.svelte';
    import type { VideoStream } from './types';
    export let stream: VideoStream;
</script>

<div class="stream-header">
    <!-- svelte-ignore a11y_consider_explicit_label -->
    <a href=".." class="back-button">
        <button>
            <svg
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path
                    d="M12 7L7 12L12 17"
                    stroke="#332425"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                />
                <path
                    d="M17 12L7.5 12"
                    stroke="#332425"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                />
            </svg>
        </button>
    </a>

    <div class="content">
        <div class="top-bar">
            <div class="stream-info">
                <h1>{stream.name}</h1>
                <div class="stream-id">{stream.id}</div>
            </div>

            <div class="right-section">
                <button class="common-action">Common Action 🧩</button>
                <button class="more-options">⋯</button>
            </div>
        </div>

        <div class="status-bar">
            <div class="status-indicators">
                <div class="twitch-icon"></div>
                <StreamStatusPill status={stream.status} />
            </div>
            <div class="date">
                {stream.created}
                <IconDate />
            </div>
        </div>
    </div>
</div>

<style>
    .stream-header {
        padding: 1rem;
        display: flex;
        flex-direction: row;
        gap: 1rem;

        .back-button {
            text-decoration: none;
            color: #333;
            width: fit-content;
            flex-shrink: 0;
            margin-top: 0.25rem;

            button {
                padding: 0;
                border: none;
                background: none;
                cursor: pointer;
            }
        }

        .content {
            flex: 1;
            .top-bar {
                display: flex;
                justify-content: space-between;
                align-items: center;
                margin-bottom: 1rem;

                .stream-info {
                    h1 {
                        margin: 0;
                        font-size: 1.5rem;
                    }

                    .stream-id {
                        color: #666;
                        font-size: 0.875rem;
                    }
                }

                .right-section {
                    display: flex;
                    gap: 0.5rem;

                    .common-action {
                        padding: 0.5rem 1rem;
                        border: 1px solid #ddd;
                        border-radius: 0.5rem;
                        background: white;
                    }

                    .more-options {
                        padding: 0.5rem;
                        border: none;
                        background: none;
                        font-size: 1.25rem;
                    }
                }
            }

            .status-bar {
                display: flex;
                justify-content: space-between;
                align-items: center;

                .status-indicators {
                    display: flex;
                    align-items: center;
                    gap: 0.5rem;
                }

                .date {
                    display: flex;
                    align-items: center;
                    gap: 0.25rem;
                }
            }
        }
    }
</style>
