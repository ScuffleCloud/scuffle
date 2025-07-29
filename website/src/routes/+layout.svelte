<script lang="ts">
    import '$styles/global.css';
    import '$styles/variables.css';
    import Navbar from '$components/left-nav/navbar.svelte';
    import TopNav from '$components/top-nav/TopNav.svelte';
    import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
    import '@fontsource-variable/archivo';
    import { browser, dev } from '$app/environment';
    import { PUBLIC_VITE_MSW_ENABLED } from '$env/static/public';
    import { enableMocking } from '$msw/setup';
    import RightNav from '$components/right-nav/RightNav.svelte';
    import { authState, initializeAuth, authAPI, type AuthResult } from '$lib/authState.svelte';
    import LoginPage from '$components/login/login-page.svelte';

    // Let's put all this in a hook to check auth status and who the user is
    $effect(() => {
        initializeAuth();
    });

    // Maybe don't need this code since we'll mock functions in a different way
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
</script>

<!-- TODO: Clean this up at some point -->
{#if mockingReady}
    {#if authState.isLoading}
        <div>Loading...</div>
    {:else if !authState.isLoggedIn}
        {@render children()}
    {:else}
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
    {/if}
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
