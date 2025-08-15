<script lang="ts">
    import type { NavItem } from '$components/types';
    import IconArrowUp from '$lib/images/icon-arrow-up.svelte';
    import IconArrowDown from '$lib/images/icon-arrow-down.svelte';

    type Props = {
        navItem: NavItem;
        isCollapsed?: boolean;
        isDropdownCollapsed?: boolean;
    };

    const { navItem, isCollapsed = false, isDropdownCollapsed = false }: Props = $props();
</script>

<div class="header" class:collapsed={isCollapsed}>
    <div class="content">
        <div class="icon">
            <navItem.icon />
        </div>
        {#if !isCollapsed}
            <div class="label">
                {navItem.label}
            </div>
        {/if}
    </div>
    {#if navItem.children && navItem.children.length > 0 && !isCollapsed}
        <div class="arrow">
            {#if isDropdownCollapsed}
                <IconArrowDown />
            {:else}
                <IconArrowUp />
            {/if}
        </div>
    {/if}
</div>

<style>
    .header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0.5rem;
        border-radius: 0.25rem;
        cursor: pointer;
        transition: justify-content 0.3s ease;

        &:hover {
            background-color: rgba(0, 0, 0, 0.05);
        }

        &.collapsed {
            justify-content: center;
        }

        .content {
            display: flex;
            align-items: center;
            gap: 0.5rem;

            .icon {
                display: flex;
                flex-shrink: 0;
            }
        }

        .label {
            font-size: 1rem;
            font-weight: 500;
            transition: opacity 0.2s ease;
            white-space: nowrap;
        }

        .arrow {
            display: flex;
            align-items: center;
            flex-shrink: 0;
        }
    }
</style>
