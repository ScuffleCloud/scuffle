<script lang="ts">
    import { Select } from 'melt/builders';
    import type { Snippet } from 'svelte';

    type SelectItem = {
        value: string;
        label: string;
        disabled?: boolean;
    };

    type Props = {
        placeholder?: string;
        items: SelectItem[];
        value?: string;
        onValueChange?: (value: string | undefined) => void;
        multiple?: boolean;
        customContent?: Snippet<[{ select: any }]>;
        customTrigger?: Snippet<[{ select: any; selectedLabel: string }]>;
    };

    let {
        value = $bindable(),
        items,
        placeholder = 'Select an option...',
        onValueChange,
        multiple = false,
        customContent,
        customTrigger,
    }: Props = $props();

    const select = new Select<string>({
        value,
        onValueChange: (newValue) => {
            value = newValue;
            onValueChange?.(newValue);
        },
        multiple: multiple as any,
    });

    const selectedLabel = $derived(
        value ? (items.find((item) => item.value === value)?.label ?? placeholder) : placeholder,
    );
</script>

<div class="select-container">
    {#if customTrigger}
        {@render customTrigger({ select, selectedLabel })}
    {:else}
        <button class="select-trigger" {...select.trigger}>
            <span class="select-label">{selectedLabel}</span>
            <span class="select-icon">▼</span>
        </button>
    {/if}

    {#if customContent}
        {@render customContent({ select })}
    {:else}
        <div class="select-content" {...select.content}>
            {#each items as item (item.value)}
                <div
                    class="select-option"
                    class:disabled={item.disabled}
                    {...select.getOption(item.value)}
                >
                    <span class="select-item-label">{item.label}</span>
                    {#if select.value === item.value}
                        <span class="select-item-indicator">✓</span>
                    {/if}
                </div>
            {/each}
        </div>
    {/if}
</div>

<style>
    .select-container {
        position: relative;
        width: 100%;
        max-width: 300px;
        margin-left: 0rem !important;
        margin-top: 0rem !important;
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
        font-size: 14px;
        line-height: 1.5;
        transition:
            border-color 0.2s,
            box-shadow 0.2s,
            background-color 0.2s;
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

    .select-trigger[data-disabled] {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .select-label {
        flex: 1;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        text-align: left;
    }

    .select-icon {
        margin-left: 8px;
        font-size: 12px;
        opacity: 0.6;
        transition: transform 0.2s;
    }

    .select-trigger[data-state='open'] .select-icon {
        transform: rotate(180deg);
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
        /* Not sure why margins are being applied automatically to my select content */
        margin-top: 0rem !important;
        margin-left: 0rem !important;
    }

    .select-option {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 8px 12px;
        cursor: pointer;
        font-size: 14px;
        border-radius: 0.25rem;
        transition: background-color 0.2s;
    }

    .select-option:hover,
    .select-option[data-highlighted] {
        background-color: #f1f5f9;
    }

    .select-option[data-selected] {
        background-color: #e6f0ff;
    }

    .select-option.disabled,
    .select-option[data-disabled] {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .select-item-label {
        flex: 1;
        text-align: left;
    }

    .select-item-indicator {
        margin-left: 8px;
        color: #0066cc;
        font-weight: bold;
    }

    /* Header styles for custom content */
    :global(.select-header) {
        padding: 0.5rem;
        color: var(--color-gray60, #64748b);
        font-size: 0.875rem;
        font-weight: 500;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        pointer-events: none; /* Prevent keyboard navigation from focusing this */
        user-select: none;
    }

    :global(.select-header .title) {
        flex: 0 0 auto;
    }

    :global(.select-header .divider) {
        width: 100%;
        height: 1px;
        background-color: #e2e8f0;
    }

    :global(.stream-info) {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    :global(.stream-id) {
        flex: 1;
    }
</style>
