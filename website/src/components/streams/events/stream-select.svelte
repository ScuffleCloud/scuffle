<script lang="ts">
    import { Select } from 'melt/builders';
    import StreamStatusPill from '$lib/shared-components/stream-status-pill.svelte';
    import type { VideoStream } from '../types';
    import IconCheckSmall from '$lib/images/icon-check-small.svelte';
    import IconSwitch_2 from '$lib/images/icon-switch-2.svelte';
    import { isNil } from 'lodash';

    type Props = {
        streams: VideoStream[];
        value?: string;
        onValueChange?: (value: string) => void;
    };

    let { streams, value = $bindable(), onValueChange }: Props = $props();

    const select = new Select<string>({
        value,
        onValueChange: (newValue) => {
            // Need to force value to not update with melt syntax when same item is selected
            if (isNil(newValue)) {
                select.value = value;
                return;
            }
            value = newValue;
            onValueChange?.(newValue);
        },
    });

    const currentStreams = $derived(streams.filter((s) => s.status === 'live'));
    const pastStreams = $derived(streams.filter((s) => s.status === 'finished'));
    const selectedStream = $derived(streams.find((stream) => stream.id === select.value));
</script>

<div class="select-container">
    <button class="select-trigger" {...select.trigger}>
        <div class="stream-info">
            {#if selectedStream}
                <StreamStatusPill status={selectedStream.status} />
                <span class="stream-id">{selectedStream.name}</span>
            {:else}
                <span class="placeholder">Select a stream...</span>
            {/if}
        </div>
        <IconSwitch_2 />
    </button>
    <div class="select-content" {...select.content}>
        <!-- Current streams section -->
        {#each currentStreams as stream, index (stream.id)}
            <div
                class="select-option"
                data-group="current"
                data-first-in-group={index === 0}
                {...select.getOption(stream.id)}
            >
                <StreamStatusPill status="live" />
                <span class="stream-id">{stream.name}</span>
                {#if select.value === stream.id}
                    <IconCheckSmall />
                {/if}
            </div>
        {/each}

        <!-- Past streams section -->
        {#each pastStreams as stream, index (stream.id)}
            <div
                class="select-option"
                data-group="past"
                data-first-in-group={index === 0}
                {...select.getOption(stream.id)}
            >
                <StreamStatusPill status="finished" />
                <span class="stream-id">{stream.id}</span>
                {#if select.value === stream.id}
                    <IconCheckSmall />
                {/if}
            </div>
        {/each}
    </div>
</div>

<style>
    .select-container {
        position: relative;
        width: 100%;
    }

    .select-trigger {
        display: inline-flex;
        align-items: center;
        padding: 0.5rem;
        background-color: white;
        border: 1px solid #e2e8f0;
        border-radius: 0.5rem;
        min-width: 400px;
        cursor: pointer;
        justify-content: space-between;
        width: 100%;
        transition:
            background-color 0.2s,
            border-color 0.2s;

        margin-top: 0rem !important;
        margin-left: 0rem !important;
    }

    .select-trigger:hover {
        background-color: #f8fafc;
        border-color: #cbd5e1;
    }

    .select-trigger:focus {
        outline: none;
        border-color: #0066cc;
        box-shadow: 0 0 0 2px rgba(0, 102, 204, 0.2);
    }

    .stream-info {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .placeholder {
        color: #64748b;
        font-style: italic;
    }

    .select-content {
        background: white;
        border-radius: 0.375rem;
        border: 1px solid #e2e8f0;
        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
        min-width: 400px;
        padding: 0.5rem;
        z-index: 50;
        max-height: 300px;
        overflow-y: auto;
        margin-left: 0rem !important;
        margin-top: 0rem !important;
    }

    .stream-id {
        flex: 1;
    }

    .select-option {
        padding: 0.5rem;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        width: 100%;
        justify-content: space-between;
        border-radius: 0.25rem;
        transition: background-color 0.2s;
        position: relative;
    }

    .select-option:hover,
    .select-option[data-highlighted] {
        background-color: #f1f5f9;
    }

    .select-option[data-selected] {
        background-color: #e6f0ff;
    }

    .select-option[data-first-in-group='true']::before {
        position: absolute;
        top: -2.5rem;
        left: 0;
        right: 0;
        padding: 0.5rem;
        color: var(--colors-gray60, #64748b);
        font-size: 0.875rem;
        font-weight: 500;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        pointer-events: none;
        user-select: none;
        background: white;
    }

    /* Custom managing of headers so keyboard navigation with Melt Select works */
    .select-option[data-group='current'][data-first-in-group='true']::before {
        content: 'Current streams';
    }

    .select-option[data-group='past'][data-first-in-group='true']::before {
        content: 'Past streams';
    }

    .select-option[data-first-in-group='true']::after {
        content: '';
        position: absolute;
        top: -1.25rem;
        height: 1px;
        background-color: #e2e8f0;
        pointer-events: none;
    }

    .select-option[data-group='current'][data-first-in-group='true']::after {
        left: 7rem;
        right: 0.5rem;
    }

    .select-option[data-group='past'][data-first-in-group='true']::after {
        left: 6.5rem;
        right: 0.5rem;
    }

    .select-option[data-first-in-group='true'] {
        margin-top: 2.5rem;
    }

    .select-option[data-group='past'][data-first-in-group='true'] {
        margin-top: 3rem;
    }
</style>
