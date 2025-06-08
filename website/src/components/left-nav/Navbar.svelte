<script lang="ts">
    import IconConfigureTab from '$lib/images/icon-configure-tab.svelte';
    import ScuffleLogo from '$lib/images/scuffle-logo.svelte';
    import NavLinks from './nav-links.svelte';
    import OrganizationInfo from './OrganizationToggle.svelte';
    import { browser } from '$app/environment';

    let isCollapsed = $state(
        browser ? localStorage.getItem('sidebar-collapsed') === 'true' : false,
    );

    // Track the original collapsed state before temporary expansion
    let originalCollapsedState = $state(isCollapsed);
    let isTemporarilyExpanded = $state(false);

    $effect(() => {
        if (browser) {
            localStorage.setItem('sidebar-collapsed', isCollapsed.toString());
        }
    });

    const toggleSidebar = () => {
        isCollapsed = !isCollapsed;
        originalCollapsedState = isCollapsed;
        isTemporarilyExpanded = false;
    };

    // Function to temporarily expand sidebar for dropdown interaction
    const handleDropdownInteraction = (shouldExpand: boolean, itemPath?: string) => {
        if (shouldExpand && isCollapsed) {
            isTemporarilyExpanded = true;
            isCollapsed = false;
        } else if (!shouldExpand && isTemporarilyExpanded) {
            // Return to original state after navigation
            isCollapsed = originalCollapsedState;
            isTemporarilyExpanded = false;
        }
    };
</script>

<nav class="sidebar" class:collapsed={isCollapsed}>
    <a class="logo-container" href="/">
        <div class="logo-container-image">
            <ScuffleLogo />
        </div>
        {#if !isCollapsed}
            <span class="logo-text">scuffle</span>
        {/if}
    </a>
    <OrganizationInfo {isCollapsed} />
    <NavLinks {isCollapsed} {handleDropdownInteraction} {isTemporarilyExpanded} />
    <button class="configure-tab-button" onclick={toggleSidebar}>
        <IconConfigureTab />
    </button>
</nav>

<style>
    .sidebar {
        width: 240px;
        height: 100vh;
        display: flex;
        flex-direction: column;
        position: sticky;
        top: 0;
        background-color: var(--colors-teal70);
        padding: 0rem 0.5rem;
        transition: width 0.25s cubic-bezier(0.4, 0, 0.2, 1);

        &.collapsed {
            width: 3.5rem;

            .logo-container {
                justify-content: center;
                padding: 1rem 0.5rem;
                gap: 0;

                .logo-container-image {
                    margin: 0;
                }
            }

            .configure-tab-button {
                right: 0.5rem;
            }
        }

        .logo-container {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            font-size: 1.5rem;
            font-weight: 800;
            padding: 1rem;
            text-transform: uppercase;
            text-decoration: none;
            transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);

            .logo-container-image {
                display: flex;
                align-items: center;
                justify-content: center;
                filter: drop-shadow(0px 2px 4px 0px rgb(0, 0, 0, 0.05));
                flex-shrink: 0;
            }

            .logo-text {
                white-space: nowrap;
                overflow: hidden;
                transition: opacity 0.15s ease-out;
            }
        }

        .configure-tab-button {
            position: absolute;
            bottom: 1rem;
            right: 1rem;
            background-color: transparent;
            border: none;
            cursor: pointer;
            padding: 0.5rem;
            border-radius: 0.5rem;
            transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);

            &:hover {
                background-color: var(--colors-teal40);
            }
        }
    }
</style>
