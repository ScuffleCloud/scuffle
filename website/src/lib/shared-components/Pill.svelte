<script lang="ts">
    import type { ThemeColors } from '$lib/theme';
    import type { Snippet } from 'svelte';
    import { getColor } from '$lib/utils';

    interface PillProps {
        children: Snippet;
        color: ThemeColors | string;
        borderColor?: ThemeColors | string;
        as?: 'button' | 'div';
        onClick?: () => void;
        class?: string;
        disabled?: boolean;
        type?: 'button' | 'submit';
        dataTestId?: string;
        width?: string;
    }

    let {
        children,
        color,
        borderColor = 'inherit',
        as = 'div',
        onClick,
        class: className = '',
        disabled = false,
        type = 'button',
        dataTestId,
        width = 'fit-content',
    }: PillProps = $props();
</script>

<svelte:element
    this={as}
    class="pill ${className}"
    style:--pill-background={getColor(color)}
    style:--pill-border-color={getColor(borderColor)}
    style:--pill-width={width}
    onclick={onClick}
    {disabled}
    role={as === 'button' ? 'button' : 'div'}
    {type}
    data-testid={dataTestId}
>
    {@render children()}
</svelte:element>

<style>
    .pill {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0.75rem 1rem;
        gap: 0.625rem;
        border-radius: 50rem;
        width: fit-content;
        cursor: pointer;
        font-size: inherit;
        background-color: var(--pill-background);
        border: 1px solid var(--pill-border-color);
        width: var(--pill-width);
        &[disabled] {
            color: var(--colors);
        }
    }
</style>
