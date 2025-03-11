<script>
    import CollapsableItem from './CollapsableItem.svelte';
    import { NAV_ITEMS } from '../consts';

    // Call CollapsableItem for each item in NAV_ITEMS
</script>

<ul class="nav-links">
    {#each NAV_ITEMS as item}
        <CollapsableItem navItem={item} />
    {/each}
</ul>

<style>
    .nav-links {
        list-style: none;
        margin: 0rem 0.25rem;
    }
</style>
