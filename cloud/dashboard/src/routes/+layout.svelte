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
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import LoginFooter from "$components/login/login-footer.svelte";
    import LoginHeader from "$components/login/login-header.svelte";
    import LoginPage from "$components/login/login-page.svelte";
    import TwoFactorPage from "$components/login/two-factor-page.svelte";
    import RightNav from "$components/right-nav/right-nav.svelte";
    import { PUBLIC_VITE_MSW_ENABLED } from "$env/static/public";
    import { authState } from "$lib/auth.svelte";
    import {
        AFTER_LOGIN_LANDING_ROUTE,
        PUBLIC_ROUTES,
    } from "$lib/consts";
    import { onMount } from "svelte";

    const auth = $derived(authState());

    onMount(() => {
        auth.initialize();
    });

    $effect(() => {
        $inspect(auth.userSessionToken);
    });

    // Let's handle routing here, or at least call it in this layout function. Maybe move elsewhere and reference here
    // We can't put any changes in authState in the +page.ts files because they don't have access to the authState
    // Until after the routing has been determined so we might as well do it here
    // Can see if there's a better way of doing this. Open to other options
    $effect(() => {
        const pathname = page.url.pathname;
        const searchParams = page.url.searchParams;

        const isPublicRoute = PUBLIC_ROUTES.includes(pathname);

        if (auth.userSessionToken.state === "loading") return;

        // Root redirect
        if (pathname === "/") {
            if (auth.userSessionToken.state === "authenticated") {
                goto(AFTER_LOGIN_LANDING_ROUTE, { replaceState: true });
            } else {
                goto("/login", { replaceState: true });
            }
            return;
        }

        // Skip routing for oauth callbacks
        if (
            searchParams.has("code")
            && (searchParams.has("state")
                || pathname.includes("magic-link"))
        ) {
            return;
        }

        // Protected routes logic. Ideally we add a redirect if this was caused by a session expiration
        if (
            !isPublicRoute
            && auth.userSessionToken.state !== "authenticated"
        ) {
            goto(`/login`);
        }

        // Public routes logic
        if (
            isPublicRoute
            && auth.userSessionToken.state === "authenticated"
        ) {
            goto(AFTER_LOGIN_LANDING_ROUTE);
        }
    });

    // Maybe don't need this code since we'll mock functions in a different way but leaving it for now
    const requireMsw = dev && browser
        && PUBLIC_VITE_MSW_ENABLED === "true";
    let mockingReady = $state(!requireMsw);

    // TODO: Remove this mocking logic eventually
    // We can probably have an organization with sample data setup in a dev env
    $effect(() => {
        if (requireMsw && !mockingReady) {
            console.log("Loading MSW");
            import("$msw/setup").then(({ enableMocking }) =>
                enableMocking()
            ).then(() => {
                mockingReady = true;
            }).catch((error) => {
                console.error("Failed to load MSW:", error);
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

    const hasPendingMfa = $derived(
        auth.userSessionToken.state === "authenticated"
            && !!auth.userSessionToken.data.mfaPending?.length,
    );
</script>

<!-- TODO: Clean this up at some point -->
{#if mockingReady}
    {#if auth.userSessionToken.state === "loading"}
        <div>Loading...</div>
    {:else if auth.userSessionToken.state === "unauthenticated"}
        <div class="login-page-container">
            <!-- TODO: Add protection to routes if not logged in -->
            <!-- This should go on each route somewhere or here might be sufficient -->
            <LoginHeader />
            <LoginPage />
            <LoginFooter />
        </div>
    {:else if hasPendingMfa}
        <div class="login-page-container">
            <LoginHeader />
            <TwoFactorPage />
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
