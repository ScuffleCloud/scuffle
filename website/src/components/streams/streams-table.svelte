<script lang="ts">
    import IconWebhook from '$lib/images/icon-webhook.svelte';
    import IconStream from '$lib/images/icon-stream.svelte';
    import type { VideoStream } from './types';
    import StreamStatusPill from '$lib/shared-components/stream-status-pill.svelte';

    export let streams: VideoStream[];

    // Map stream statuses to text that gets displayed here
    const iconMap = {
        live: IconWebhook,
        finished: IconStream,
    };
</script>

<div class="table-wrapper">
    <table class="streams-table">
        <thead>
            <tr>
                <th class="status-column">status</th>
                <th class="name-column">stream name</th>
                <th class="id-column">stream id</th>
                <th class="created-column">created</th>
            </tr>
        </thead>
        <tbody>
            {#each streams as stream (stream.id)}
                <tr class="stream-row">
                    <td colspan="4" class="row-cell">
                        <a href="streams/{stream.id}" class="row-link">
                            <div class="row-content">
                                <div class="status-column-content">
                                    <div class="status-wrapper">
                                        <svelte:component this={iconMap[stream.status]} />
                                        <StreamStatusPill status={stream.status} />
                                    </div>
                                </div>
                                <div class="name-column-content">
                                    {stream.name}
                                </div>
                                <div class="id-column-content">
                                    {stream.id}
                                </div>
                                <div class="created-column-content">
                                    {stream.created}
                                </div>
                            </div>
                        </a>
                    </td>
                </tr>
            {/each}
        </tbody>
    </table>
</div>

<style>
    .table-wrapper {
        overflow: hidden;
    }

    .streams-table {
        width: 100%;
        border-collapse: separate;
        border-spacing: 0 0.25rem;
        background: transparent;
        border: none;

        thead {
            tr {
                background-color: #f9f5f2;
            }

            th {
                text-align: left;
                padding: 0.75rem 1rem;
                font-size: 0.875rem;
                color: #666;
                font-weight: 500;
            }
        }

        .stream-row {
            background: var(--colors-teal30);
            transition: background-color 0.1s ease-in-out;
        }

        .row-cell {
            padding: 0;
            background: white;
            border-top-left-radius: 0.5rem;
            border-bottom-left-radius: 0.5rem;
            border-top-right-radius: 0.5rem;
            border-bottom-right-radius: 0.5rem;
        }

        .row-link {
            display: block;
            padding: 1rem;
            text-decoration: none;
            color: inherit;
            transition: background-color 0.1s ease-in-out;
            border-radius: 0.5rem;

            &:hover {
                background-color: rgba(0, 0, 0, 0.02);
            }

            &:focus {
                outline: 2px solid #007acc;
                outline-offset: -2px;
                background-color: rgba(0, 122, 204, 0.1);
            }
        }

        .row-content {
            display: flex;
            align-items: center;
        }

        .status-column-content {
            width: 15%;
        }

        .status-wrapper {
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }

        .name-column-content {
            width: 30%;
            font-weight: 500;
        }

        .id-column-content {
            width: 30%;
            color: #666;
            font-family: monospace;
            font-size: 0.875rem;
        }

        .created-column-content {
            width: 25%;
        }

        /* Column alignment for headers */
        .status-column {
            width: 15%;
        }

        .name-column {
            width: 30%;
        }

        .id-column {
            width: 30%;
        }

        .created-column {
            width: 25%;
        }
    }
</style>
