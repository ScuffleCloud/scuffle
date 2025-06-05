<script lang="ts">
    import { NAV_ITEMS } from '$components/left-nav/consts.svelte';
    import { useUser } from '$lib/useUser';
    import NavItemDropdown from './nav-item-dropdown.svelte';
    import NavItemBase from './nav-item-base.svelte';

    type Props = {
        isCollapsed?: boolean;
    };

    const { isCollapsed = false }: Props = $props();

    const { currentOrganization, currentProject } = useUser();

    const basePath = $derived(
        `/organizations/${$currentOrganization?.slug}/projects/${$currentProject?.slug}`,
    );

    const navItemsWithPaths = $derived(
        NAV_ITEMS.map((item) => ({
            ...item,
            path: `${basePath}${item.path}`,
        })),
    );
</script>

<ul class="nav-links" class:collapsed={isCollapsed}>
    {#each navItemsWithPaths as item}
        {#if item.children && !isCollapsed}
            <NavItemDropdown navItem={item} {isCollapsed} />
        {:else}
            <a href={item.path} title={isCollapsed ? item.label : ''}>
                <NavItemBase navItem={item} {isCollapsed} />
            </a>
        {/if}
    {/each}
</ul>

<style>
    .nav-links {
        list-style: none;
        margin: 0rem 0rem;
        padding: 0rem;
        border-radius: 0rem;
        display: flex;
        flex-direction: column;
        gap: 0.25rem;

        a {
            text-decoration: none;
        }

        &.collapsed a {
            display: flex;
            justify-content: center;
        }
    }
</style>
