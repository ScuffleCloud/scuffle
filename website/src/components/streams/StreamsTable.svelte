<script lang="ts">
    import IconWebhook from '$lib/images/IconWebhook.svelte';
    import IconStream from '$lib/images/IconStream.svelte';
    import type { VideoStream } from './types';
    import { goto } from '$app/navigation';

    export let streams: VideoStream[];

    // Map stream statuses to text that gets displayed here
    const statusMap = {
        live: {
            text: '• Live',
            icon: IconWebhook,
        },
        finished: {
            text: 'Finished',
            icon: IconStream,
        },
    };

    function handleRowClick(id: string) {
        goto(`/streams/${id}`);
    }
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
                <tr
                    class="stream-row"
                    on:click={() => handleRowClick(stream.id)}
                    on:keydown={(e) => e.key === 'Enter' && handleRowClick(stream.id)}
                    role="link"
                    tabindex="0"
                >
                    <td class="status-column">
                        <div class="status-wrapper">
                            <svelte:component this={statusMap[stream.status].icon} />
                            <span class={`status-badge ${stream.status}`}>
                                {statusMap[stream.status].text}
                            </span>
                        </div>
                    </td>
                    <td class="name-column">{stream.name}</td>
                    <td class="id-column">{stream.id}</td>
                    <td class="created-column">{stream.created}</td>
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
            background-color: white;
            background: var(--color-teal30);
            cursor: pointer;
            transition:
                transform 0.1s ease-in-out,
                background-color 0.1s ease-in-out;

            &:active {
                transform: translateY(0);
            }

            td {
                padding: 1rem;
                background: white;

                &:first-child {
                    border-top-left-radius: 0.5rem;
                    border-bottom-left-radius: 0.5rem;
                }

                &:last-child {
                    border-top-right-radius: 0.5rem;
                    border-bottom-right-radius: 0.5rem;
                }
            }
        }

        .status-column {
            width: 15%;

            .status-wrapper {
                display: flex;
                align-items: center;
                gap: 0.5rem;
            }

            .status-badge {
                display: inline-block;
                padding: 0.25rem 0.5rem;
                border-radius: 0.25rem;
                font-size: 0.875rem;
                font-weight: 500;

                &.live {
                    padding: 4px 9px;
                    color: #7c0505;
                    background-color: #fed6d6;
                    border-radius: 100rem;
                }

                &.finished {
                    color: #666;
                    background-color: #f0f0f0;
                }
            }
        }

        .name-column {
            width: 30%;
            font-weight: 500;
        }

        .id-column {
            width: 30%;
            color: #666;
            font-family: monospace;
            font-size: 0.875rem;
        }

        .created-column {
            width: 25%;
        }
    }
</style>
