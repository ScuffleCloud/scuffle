<script lang="ts">
    import "$styles/global.css";
    import "$styles/variables.css";
    import Navbar from "$components/left-nav/navbar.svelte";
    import TopNav from "$components/top-nav/top-nav.svelte";
    import {
        QueryClient,
        QueryClientProvider,
    } from "@tanstack/svelte-query";
    import "@fontsource-variable/archivo";
    import { browser, dev } from "$app/environment";
    import LoginFooter from "$components/login/login-footer.svelte";
    import LoginHeader from "$components/login/login-header.svelte";
    import LoginPage from "$components/login/login-page.svelte";
    import RightNav from "$components/right-nav/right-nav.svelte";
    import { PUBLIC_VITE_MSW_ENABLED } from "$env/static/public";
    import {
        authAPI,
        type AuthResult,
        authState,
        initializeAuth,
    } from "$lib/authState.svelte";
    import { enableMocking } from "$msw/setup";

    // Let's put all this in a hook to check auth status and who the user is
    $effect(() => {
        initializeAuth();
    });

    // Maybe don't need this code since we'll mock functions in a different way but leaving it for now
    const requireMsw = dev && browser
        && PUBLIC_VITE_MSW_ENABLED === "true";
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
        <div class="login-page-container">
            <!-- TODO: Add protection to routes if not logged in -->
            <LoginHeader />
            <LoginPage />
            <LoginFooter />
        </div>
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

    .login-page-container {
      min-height: 100vh;
      display: flex;
      align-items: center;
      justify-content: center;
      background-color: #f5f3f0;
      padding: 2rem;
      flex-direction: column;
    }
</style>
