<!-- nav-item-base.svelte -->
<script lang="ts">
    import type { NavItem } from '$components/types';
    import IconArrowUp from '$lib/images/icon-arrow-up.svelte';
    import IconArrowDown from '$lib/images/icon-arrow-down.svelte';

    type Props = {
        navItem: NavItem;
        isCollapsed?: boolean;
    };

    const { navItem, isCollapsed = false }: Props = $props();
</script>

<div class="header">
    <div class="content">
        <navItem.icon />
        <div class="label">
            {navItem.label}
        </div>
    </div>
    {#if navItem.children && navItem.children.length > 0}
        <div class="arrow">
            {#if isCollapsed}
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
        padding: 0.75rem 0.38rem;
        border-radius: 0.25rem;
        cursor: pointer;

        &:hover {
            background-color: rgba(0, 0, 0, 0.05);
        }

        .content {
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }

        .label {
            font-size: 1rem;
            font-weight: 500;
        }

        .arrow {
            display: flex;
            align-items: center;
        }
    }
</style>
