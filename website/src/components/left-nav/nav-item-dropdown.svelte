<script lang="ts">
    import type { NavItem } from '$components/types';
    import { page } from '$app/state';
    import { Collapsible } from 'melt/builders';
    import NavItemBase from './nav-item-base.svelte';

    const collapsible = new Collapsible();

    type Props = {
        navItem: NavItem;
    };

    const { navItem }: Props = $props();
</script>

<div class="root">
    <div {...collapsible.trigger}>
        <NavItemBase {navItem} isCollapsed={!collapsible.open} />
    </div>
    {#if collapsible.open}
        <div {...collapsible.content}>
            <div class="collapsible">
                {#each navItem.children ?? [] as child}
                    <a
                        class="item"
                        href={child.path}
                        class:active={page.url.pathname.includes(child.path)}
                    >
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

        .collapsible {
            display: flex;
            flex-direction: column;
            gap: 0.25rem;

            .item {
                padding: 0.75rem 2rem;
                border-radius: 0.25rem;
                text-decoration: none;
                color: inherit;
                display: block;

                span {
                    font-size: 1rem;
                }

                &:hover {
                    background-color: rgba(0, 0, 0, 0.05);
                }

                &.active {
                    background-color: #fed48b4d;
                    position: relative;

                    &::before {
                        content: '';
                        position: absolute;
                        left: 1rem;
                        top: 20%;
                        bottom: 20%;
                        width: 3px;
                        background-color: #f9a825;
                        border-radius: 1.5px;
                    }
                }
            }
        }
    }
</style>
