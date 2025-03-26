<script lang="ts">
    import type { NavItem } from '$components/types';

    type Props = {
        navItem: NavItem;
        isCollapsible?: boolean;
    };

    const { navItem, isCollapsible }: Props = $props();
</script>

<div class="header" class:collapsible={isCollapsible}>
    <navItem.icon />
    <div class="label">
        {navItem.label}
    </div>
</div>

<style>
    .header {
        display: flex;
        align-items: center;
        text-align: center;
        padding: 0.75rem 0.38rem;
        border-radius: 0.25rem;
        gap: 0.5rem;

        &:hover {
            background-color: rgba(0, 0, 0, 0.05);
        }

        &.collapsible {
            cursor: pointer;
        }

        .label {
            font-size: 1rem;
            font-weight: 500;
        }
    }
</style>
