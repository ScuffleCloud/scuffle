<script lang="ts">
    import { afterNavigate } from "$app/navigation";
    import { NAV_ITEMS } from "$components/left-nav/consts.svelte";
    import type { NavItem } from "$components/types";
    import NavItemBase from "./nav-item-base.svelte";
    import NavItemDropdown from "./nav-item-dropdown.svelte";

    type Props = {
        isCollapsed?: boolean;
        handleDropdownInteraction?: (
            shouldExpand: boolean,
            itemPath?: string,
        ) => void;
        isTemporarilyExpanded?: boolean;
    };

    const {
        isCollapsed = false,
        handleDropdownInteraction,
        isTemporarilyExpanded = false,
    }: Props = $props();

    const currentOrganization = {
        slug: "org-1",
    };
    const currentProject = {
        slug: "proj-1",
    };

    const basePath = $derived(
        `/organizations/${currentOrganization?.slug}/projects/${currentProject?.slug}`,
    );

    const navItemsWithPaths = $derived(
        NAV_ITEMS.map((item) => {
            // Not relative to base path
            if (!item.path.startsWith("/")) {
                return {
                    ...item,
                    path: `/${item.path}`,
                };
            }
            return {
                ...item,
                path: `${basePath}${item.path}`,
            };
        }),
    );

    let shouldOpenDropdown = $state<string | null>(null);

    const handleDropdownClick = (event: MouseEvent, item: NavItem) => {
        if (isCollapsed && item.children && handleDropdownInteraction) {
            event.preventDefault();
            shouldOpenDropdown = item.path;
            handleDropdownInteraction(true, item.path);
        }
    };

    afterNavigate(() => {
        if (handleDropdownInteraction) {
            handleDropdownInteraction(false);
        }
        shouldOpenDropdown = null;
    });
</script>

<ul class="nav-links" class:collapsed={isCollapsed}>
    {#each navItemsWithPaths as item (item.id)}
        {#if item.children && !isCollapsed}
            <NavItemDropdown
                navItem={item}
                {isCollapsed}
                shouldOpen={shouldOpenDropdown === item.path && isTemporarilyExpanded}
            />
        {:else}
            <a
                href={item.path}
                title={isCollapsed ? item.label : ""}
                onclick={(e) => handleDropdownClick(e, item)}
            >
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
