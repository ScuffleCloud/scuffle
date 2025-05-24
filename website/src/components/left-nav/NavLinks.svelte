<script>
    import { NAV_ITEMS } from '$components/left-nav/consts.svelte';
    import { useUser } from '$lib/useUser';
    import NavItemHeader from './NavItemHeader.svelte';

    // Get the current organization and project
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

<ul class="nav-links">
    {#each navItemsWithPaths as item}
        <a href={item.path}>
            <NavItemHeader navItem={item} />
        </a>
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
