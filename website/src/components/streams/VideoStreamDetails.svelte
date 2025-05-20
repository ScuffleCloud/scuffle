<script lang="ts">
    import type { Streamed } from '$lib/types';
    import { Tabs } from 'melt/builders';
    import type { VideoStream } from './types';
    import { page } from '$app/state';
    import StreamDetailsHeader from './stream-detail-header.svelte';
    import { pushState, replaceState } from '$app/navigation';
    import EventsTab from './events/events-tab.svelte';
    import OverviewTab from './overview/overview-tab.svelte';

    type Props = {
        streamedData: Streamed<VideoStream>;
    };
    const { streamedData }: Props = $props();
    const tabIds = ['overview', 'events', 'assets'];
    let currentTab = $state('overview');
    let initialStateSet = $state(false);
    // For my usage - https://sveltekit.io/blog/shallow-routing
    // This solution may change to using layouts in the future depending on the data loading
    // Will just do it with states for now
    $effect(() => {
        console.log('trigger here');
        if (!page.params.tab && !initialStateSet) {
            initialStateSet = true;
            replaceState(`${page.params.id}/overview`, {
                ...page.state,
                activeTab: 'overview',
            });
        }
    });
    const tabs = new Tabs({
        // TODO: refactor put a getter here
        value: currentTab,
        onValueChange: (value) => {
            currentTab = value;
            pushState(`${value}`, {
                ...page.state,
                activeTab: value,
            });
        },
    });
</script>

<!-- TODO: Add skeleton loading state -->
{#await streamedData}
    <div class="loading">Loading...</div>
{:then stream}
    <div class="stream-details-container">
        <StreamDetailsHeader {stream} />
        <div class="tabs-container">
            <div class="tabs-list-container" {...tabs.triggerList}>
                {#each tabIds as id}
                    <button class="tab-trigger" {...tabs.getTrigger(id)}>
                        {id.charAt(0).toUpperCase() + id.slice(1)}
                    </button>
                {/each}
            </div>

            {#each tabIds as id}
                <div class="tab-content" {...tabs.getContent(id)}>
                    {#if id === 'overview'}
                        <div class="overview-content">
                            <h3>Stream Overview</h3>
                            <OverviewTab />
                        </div>
                    {:else if id === 'events'}
                        <div class="events-content">
                            <h3>Events</h3>
                            <EventsTab />
                        </div>
                    {:else if id === 'assets'}
                        <div class="assets-content">
                            <h3>Stream Assets</h3>
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
    </div>
{:catch error}
    <div class="error">Error: {error.message}</div>
{/await}

<style>
    .stream-details-container {
        display: flex;
        flex-direction: column;
        gap: 1rem;

        .tabs-container {
            .tabs-list-container {
                display: flex;
                gap: 1rem;
                border-bottom: 1px solid #e2e8f0;
                margin-bottom: 1rem;

                .tab-trigger {
                    padding: 0.75rem 1rem;
                    border: none;
                    background: none;
                    color: #64748b;
                    cursor: pointer;
                    font-weight: 500;
                    border-bottom: 2px solid transparent;
                    transition: all 0.2s;

                    &[data-selected] {
                        color: #0f172a;
                        border-bottom-color: #0f172a;
                    }

                    &:hover:not([data-selected]) {
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

                .events-content {
                    position: relative;
                }

                h3 {
                    margin: 0 0 1rem 0;
                    font-size: 1.25rem;
                    font-weight: 600;
                }

                &[hidden] {
                    display: none;
                }
            }
        }
    }
</style>
