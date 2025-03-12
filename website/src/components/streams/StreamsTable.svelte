<script lang="ts">
    import ChevronDown from '$lib/images/ChevronDown.svelte';
    import type { VideoStream } from './types';
    import { Accordion, type AccordionItem } from 'melt/builders';
    import { slide } from 'svelte/transition';

    export let streams: VideoStream[];

    type Item = AccordionItem<{
        videoStream: VideoStream;
    }>;

    const accordionItems: Item[] = streams.map((stream) => ({
        videoStream: stream,
        id: stream.id,
    }));

    const accordion = new Accordion();
</script>

<div class="streams-table">
    <div class="table-header">
        <div class="column status-column">status</div>
        <div class="column name-column">stream name</div>
        <div class="column id-column">stream id</div>
        <div class="column created-column">created</div>
    </div>

    <div class="table-body" {...accordion.root}>
        {#each accordionItems as accordionItem (accordionItem.id)}
            {@const item = accordion.getItem(accordionItem)}
            {@const { status, name, id, created } = accordionItem.videoStream}
            <div class="stream-row" {...item.heading} {...item.trigger}>
                <div class="column status-column">
                    {#if status === 'Live'}
                        <span class="status-badge live">• Live</span>
                    {:else}
                        <span class="status-badge finished">Finished</span>
                    {/if}
                </div>
                <div class="column name-column">{name}</div>
                <div class="column id-column">{id}</div>
                <div class="column created-column">
                    {created}
                    <button class="expand-button">
                        <ChevronDown />
                    </button>
                </div>
            </div>
            {#if item.isExpanded}
                <div class="stream-preview" {...item.content} transition:slide>
                    <div class="preview-container">
                        <div class="play-button">
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                width="24"
                                height="24"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            >
                                <polygon points="5 3 19 12 5 21 5 3"></polygon>
                            </svg>
                        </div>
                        <div class="duration">64.62</div>
                    </div>
                </div>
            {/if}
        {/each}
    </div>
</div>

<style>
    .streams-table {
        background-color: #fff;
        border-radius: 0.75rem;
        overflow: hidden;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);

        .table-header {
            display: flex;
            background-color: #f9f5f2;
            padding: 0.75rem 1rem;
            font-size: 0.875rem;
            color: #666;
            font-weight: 500;
            border-bottom: 1px solid #eee;

            .column {
                cursor: pointer;

                &:hover {
                    color: #000;
                }
            }
        }

        .table-body {
            .stream-row {
                display: flex;
                padding: 1rem;
                border-bottom: 1px solid #eee;
                align-items: center;
                transition: background-color 0.2s;

                &:hover {
                    background-color: #fafafa;
                }
            }

            .stream-preview {
                padding: 1rem;
                background-color: #f9f9f9;
                border-bottom: 1px solid #eee;

                .preview-container {
                    background-color: #eee;
                    height: 100px;
                    border-radius: 0.5rem;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    position: relative;

                    .play-button {
                        background: none;
                        border: none;
                        cursor: pointer;
                        color: #666;

                        &:hover {
                            color: #000;
                        }
                    }

                    .duration {
                        position: absolute;
                        top: 0.5rem;
                        right: 0.5rem;
                        background-color: rgba(255, 0, 0, 0.8);
                        color: white;
                        padding: 0.25rem 0.5rem;
                        border-radius: 0.25rem;
                        font-size: 0.75rem;
                        font-weight: 600;
                    }
                }
            }
        }

        .column {
            &.status-column {
                width: 15%;

                .status-badge {
                    display: inline-block;
                    padding: 0.25rem 0.5rem;
                    border-radius: 0.25rem;
                    font-size: 0.875rem;
                    font-weight: 500;

                    &.live {
                        color: #e53935;
                    }

                    &.finished {
                        color: #666;
                        background-color: #f0f0f0;
                    }
                }
            }

            &.name-column {
                width: 30%;
                font-weight: 500;
            }

            &.id-column {
                width: 30%;
                color: #666;
                font-family: monospace;
                font-size: 0.875rem;
            }

            &.created-column {
                width: 25%;
                display: flex;
                justify-content: space-between;
                align-items: center;

                .expand-button {
                    background: none;
                    border: none;
                    cursor: pointer;
                    color: #888;

                    &:hover {
                        color: #000;
                    }
                }
            }
        }
    }
</style>
