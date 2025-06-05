<script lang="ts">
    import { theme } from '$lib/theme';
    import '$styles/global.css';
    import Navbar from '$components/left-nav/navbar.svelte';
    import TopNav from '$components/top-nav/TopNav.svelte';
    import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
    import '@fontsource-variable/archivo';
    import { browser, dev } from '$app/environment';
    import { PUBLIC_VITE_MSW_ENABLED } from '$env/static/public';
    import { enableMocking } from '$msw/setup';
    import RightNav from '$components/right-nav/RightNav.svelte';

    const requireMsw = dev && browser && PUBLIC_VITE_MSW_ENABLED === 'true';
    let mockingReady = $state(!requireMsw);

    $effect(() => {
        if (requireMsw && !mockingReady) {
            enableMocking().then(() => {
                mockingReady = true;
            });
        }
    });
    const { children } = $props();
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: {
                enabled: browser,
            },
        },
    });

    const rootCssVariables = Object.entries(theme.colors)
        .map(([key, value]) => `--colors-${key}: ${value}`)
        .join(';');

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const inlineStyleForRoot = `:root { ${rootCssVariables} }`;
</script>

<!-- Otherwise dynamically generated CSS variables aren't correctly available in the app -->
<svelte:head>
    {@html `<style>${inlineStyleForRoot}</style>`}
</svelte:head>

{#if mockingReady}
    <div class="app">
        <QueryClientProvider client={queryClient}>
            <Navbar />
            <main>
                <TopNav />
                <div class="content">
                    <div class="main-content">
                        {@render children()}
                    </div>
                    <RightNav />
                </div>
            </main>
        </QueryClientProvider>
    </div>
{:else}
    <div>Error loading mocks...</div>
{/if}

<style>
    .app {
        position: relative;
        display: flex;
        min-height: 100vh;
        background-color: var(--colors-light100);
    }

    main {
        position: relative;
        flex: 1;
        display: flex;
        flex-direction: column;
        box-sizing: border-box;
        width: 100%;
        /* So content resizes properly */
        min-width: 0;

        .content {
            display: flex;
            flex-direction: row;
            padding: 2rem;

            .main-content {
                flex: 1;
                width: 100%;
                /* So content resizes properly */
                min-width: 0;
            }
        }
    }
</style>
