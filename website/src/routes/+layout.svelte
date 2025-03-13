<script lang="ts">
    import { theme } from '$lib/theme';
    import '$styles/global.css';
    import Navbar from '$components/left-nav/Navbar.svelte';
    import TopNav from '$components/top-nav/TopNav.svelte';
    import '@fontsource-variable/archivo';
    const { children } = $props();
    import { browser, dev } from '$app/environment';
    import { PUBLIC_VITE_MSW_ENABLED } from '$env/static/public';
    import { enableMocking } from '$msw/setup';

    const requireMsw = dev && browser && PUBLIC_VITE_MSW_ENABLED === 'true';
    let mockingReady = $state(!requireMsw);

    $effect(() => {
        if (requireMsw && !mockingReady) {
            enableMocking().then(() => {
                mockingReady = true;
            });
        }
    });

    const cssVariables = Object.entries(theme.colors)
        .map(([key, value]) => `--color-${key}: ${value}`)
        .join(';');
</script>

{#if mockingReady}
    <div class="app" style={cssVariables}>
        <Navbar />
        <main>
            <TopNav />
            <div class="content">
                {@render children()}
            </div>
        </main>
    </div>
{:else}
    <div>Error loading mocks...</div>
{/if}

<style>
    .app {
        position: relative;
        display: flex;
        min-height: 100vh;
        background-color: var(--color-light100);
    }

    main {
        position: relative;
        flex: 1;
        display: flex;
        flex-direction: column;
        box-sizing: border-box;

        .content {
            padding: 2rem;
        }
    }
</style>
