<script lang="ts">
    import type { NavItem } from '$components/types';

    type Props = {
        navItem: NavItem;
    };

    const { navItem }: Props = $props();
</script>

<div class="header">
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
        .label {
            font-size: 1rem;
            font-weight: 500;
        }
    }
</style>
