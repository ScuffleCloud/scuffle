<script>
    import CollapsableItem from './CollapsableItem.svelte';
    import { NAV_ITEMS } from '$components/consts.svelte';
    import NavItemHeader from './NavItemHeader.svelte';
</script>

<ul class="nav-links">
    {#each NAV_ITEMS as item}
        {#if item.children}
            <CollapsableItem navItem={item} />
        {:else}
            <a href={item.path}>
                <NavItemHeader navItem={item} isCollapsible />
            </a>
        {/if}
    {/each}
</ul>

<style>
    .nav-links {
        list-style: none;
        margin: 0rem 0rem;
        padding: 0rem 0.5rem;
        a {
            text-decoration: none;
        }
    }
</style>
