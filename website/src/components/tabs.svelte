<!-- tabs.svelte -->
<script lang="ts">
    import { page } from '$app/state';
    import type { Snippet } from 'svelte';

    import type { Component } from 'svelte';

    export type Tab = {
        id: string;
        label: string;
        icon?: Component;
    };

    type Props = {
        tabs: Tab[];
        baseUrl: string;
        defaultTab?: string;
        children: Snippet;
    };

    const { tabs, baseUrl, defaultTab = tabs[0]?.id, children }: Props = $props();

    const currentTab = $derived.by(() => {
        const pathname = page.url.pathname;

        if (pathname.startsWith(baseUrl)) {
            const remainder = pathname.slice(baseUrl.length);
            const firstSegment = remainder.split('/')[0];
            return tabs.find((tab) => tab.id === firstSegment)?.id || defaultTab;
        }

        return defaultTab;
    });
</script>

<div class="tabs-container">
    <div class="tabs-list-container">
        {#each tabs as tab}
            <a
                href={`${baseUrl}${tab.id}`}
                class="tab-trigger"
                data-selected={currentTab === tab.id}
            >
                {#if tab.icon}
                    <tab.icon />
                {/if}
                <span class="tab-label">{tab.label}</span>
            </a>
        {/each}
    </div>

    <div class="tab-content">
        {@render children()}
    </div>
</div>

<style>
    .tabs-container {
        .tabs-list-container {
            display: flex;
            gap: 1rem;
            border-bottom: 1px solid var(--color-gray40);
            margin-bottom: 1rem;

            .tab-trigger {
                display: flex;
                align-items: center;
                gap: 0.5rem;
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

                .tab-label {
                    white-space: nowrap;
                }
            }
        }

        .tab-content {
            padding: 1rem 0;
        }
    }
</style>
