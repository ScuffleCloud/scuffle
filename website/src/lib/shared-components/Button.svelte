<script lang="ts">
    import type { Snippet } from 'svelte';
    import type { HTMLButtonAttributes, HTMLAnchorAttributes } from 'svelte/elements';

    // different types of buttons:
    // - links looking like buttons
    // - real buttons with onclick
    //
    // different styles:
    // - icon only (square), text only, icon + text
    // - primary, secondary, no bg
    // - disabled, active, hover (focus)

    type Props = {
        // href decides if this is a link looking like button or if it's a real button
        href?: string | null;
        primary?: boolean;
        secondary?: boolean;
        big?: boolean;
        submit?: boolean;
        hideOnMobile?: boolean;
        hideOnDesktop?: boolean;
        children?: Snippet;
        icon?: Snippet;
        iconRight?: Snippet;
    } & HTMLButtonAttributes &
        HTMLAnchorAttributes;

    let {
        href = null,
        primary = false,
        secondary = false,
        big = false,
        submit = false,
        hideOnMobile = false,
        hideOnDesktop = false,
        children,
        icon,
        iconRight,
        ...restProps
    }: Props = $props();
</script>

{#if href}
    <a
        {href}
        {...restProps}
        class="button"
        class:primary
        class:secondary
        class:big
        class:has-text={children}
        class:icon-only={!children && (icon || iconRight)}
        class:icon-left={icon}
        class:icon-right={iconRight}
        class:hide-on-mobile={hideOnMobile}
        class:hide-on-desktop={hideOnDesktop}
    >
        {@render icon?.()}
        {@render children?.()}
        {@render iconRight?.()}
    </a>
{:else}
    <button
        type={submit ? 'submit' : 'button'}
        {...restProps}
        class="button"
        class:primary
        class:secondary
        class:big
        class:has-text={children}
        class:icon-only={!children && (icon || iconRight)}
        class:icon-left={icon}
        class:icon-right={iconRight}
        class:hide-on-mobile={hideOnMobile}
        class:hide-on-desktop={hideOnDesktop}
    >
        {@render icon?.()}
        {@render children?.()}
        {@render iconRight?.()}
    </button>
{/if}

<style>
    a,
    button {
        cursor: pointer;
        font: inherit;
        border: 1px solid transparent;
        color: var(--text);
        font-weight: 600;
        padding: 0.5rem;
        border-radius: 0.5rem;
        transition: background-color 0.1s;
        white-space: nowrap;
        user-select: none;
        font-size: 0.875rem;

        display: flex;
        align-items: center;
        gap: 0.5rem;

        &.has-text {
            padding: 0.5rem 1rem;

            &.icon-left {
                padding-left: 0.75rem;
            }

            &.icon-right {
                padding-right: 0.75rem;
            }
        }

        &.big {
            padding: 0.75rem;
            font-weight: 500;

            gap: 0.75rem;
        }

        &.secondary {
            background-color: var(--secondary);
            color: var(--secondary-text);
            border-color: var(--secondary-border);

            &:disabled {
                background-color: var(--secondary-disabled);
            }
        }

        &:disabled {
            cursor: not-allowed;
            color: var(--text-light);
        }

        &:not(:disabled) {
            &:hover,
            &:focus-visible {
                text-decoration: none;
                background-color: var(--secondary-hover);
            }

            &:active {
                background-color: var(--secondary-active);
            }
        }

        &.primary {
            background-color: var(--primary);
            color: var(--primary-text);
            border-color: var(--primary-border);

            &:disabled {
                background-color: var(--primary-disabled);
            }

            &:not(:disabled) {
                &:hover,
                &:focus-visible {
                    background-color: var(--primary-hover);
                }

                &:active {
                    background-color: var(--primary-active);
                }
            }
        }
    }
</style>
