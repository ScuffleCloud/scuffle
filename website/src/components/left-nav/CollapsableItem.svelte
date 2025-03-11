<script lang="ts">
    import type { NavItem } from '$components/types';

    import { Collapsible } from 'melt/builders';

    const collapsible = new Collapsible();
    const { navItem } = $props<{ navItem: NavItem }>();
</script>

<div class="root">
    <div class="header" {...collapsible.trigger}>
        <span class="label">{navItem.label}</span>
    </div>
    {#if collapsible.open}
        <div {...collapsible.content}>
            <div class="collapsible">
                {#each navItem.children as child}
                    <a class="item" href={child.path}>
                        <span>{child.label}</span>
                    </a>
                {/each}
            </div>
        </div>
    {/if}
</div>

<style>
    .root {
        width: 100%;

        .header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            cursor: pointer;
            padding: 0.75rem 0rem;
            border-radius: 0.25rem;

            .label {
                font-size: 1rem;
                line-height: 1.5rem;
            }
        }
    }

    .collapsible {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;

        .item {
            padding: 0.75rem;
            border-radius: 0.25rem;
        }

        .item span {
            font-size: 1rem;
        }
    }
</style>
