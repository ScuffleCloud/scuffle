<script lang="ts">
    import type { VideoStream } from '$components/streams/types';
    import type { Streamed } from '$lib/types';
    import StreamDetailsHeader from '$components/streams/stream-detail-header.svelte';
    import { page } from '$app/state';
    import type { Snippet } from 'svelte';

    type ChildProps = {
        data: VideoStream;
    };

    type Props = {
        data: {
            stream: Streamed<VideoStream>;
        };
        children: Snippet<[ChildProps]>;
    };

    const { data: pageData, children }: Props = $props();

    const tabs = [
        { id: 'overview', label: 'Overview' },
        { id: 'events', label: 'Events' },
        { id: 'assets', label: 'Assets' },
    ];

    const currentTab = $derived(page.url.pathname.split('/').pop());
</script>

<div class="page-bg">
    {#await pageData.stream}
        <div class="loading">Loading...</div>
    {:then stream}
        <StreamDetailsHeader {stream} />
        <div class="tabs-container">
            <div class="tabs-list-container">
                {#each tabs as tab}
                    <a
                        href={`/streams/${stream.id}/${tab.id}`}
                        class="tab-trigger"
                        data-selected={currentTab === tab.id}
                    >
                        {tab.label}
                    </a>
                {/each}
            </div>

            <div class="tab-content">
                {@render children({ data: stream })}
            </div>
        </div>
    {:catch error}
        <div class="error">Error: {error.message}</div>
    {/await}
</div>

<style>
    .page-bg {
        background-color: var(--colors-light100);
        margin: 0 auto;
        width: 100%;
        max-width: 1200px;
        padding: 2rem;
    }

    .tabs-container {
        .tabs-list-container {
            display: flex;
            gap: 1rem;
            border-bottom: 1px solid var(--color-gray40);
            margin-bottom: 1rem;

            .tab-trigger {
                padding: 0.75rem 1rem;
                border: none;
                background: none;
                color: var(--color-yellow90);
                font-size: 1rem;
                font-weight: 700;
                line-height: 1.5rem;
                cursor: pointer;
                font-weight: 500;
                border-bottom: 2px solid transparent;
                transition: all 0.2s;
                text-decoration: none;

                &[data-selected='true'] {
                    /* Need to give radius to this border or just make a new div for the styling */
                    /* border-radius: 0.125rem 0.125rem 0rem 0rem; */
                    border-bottom-color: var(--color-yellow40);
                }

                &:hover:not([data-selected='true']) {
                    color: #334155;
                }

                &:focus-visible {
                    outline: 2px solid #3b82f6;
                    outline-offset: 2px;
                    border-radius: 0.25rem;
                }
            }
        }

        .tab-content {
            padding: 1rem 0;
        }
    }
</style>
