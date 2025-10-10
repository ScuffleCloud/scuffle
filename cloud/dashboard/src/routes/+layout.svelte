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
    import { browser } from "$app/environment";
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import TwoFactorPage from "$components/login-two-factor/two-factor-page.svelte";
    import LoginFooter from "$components/login/login-footer.svelte";
    import LoginHeader from "$components/login/login-header.svelte";
    import LoginPage from "$components/login/login-page.svelte";
    import RightNav from "$components/right-nav/right-nav.svelte";
    import { authState } from "$lib/auth.svelte";
    import {
        LANDING_ROUTE,
        LOGIN_ROUTES,
        OAUTH_CALLBACK_ROUTES,
    } from "$lib/consts";

    const auth = $derived(authState());

    const isOAuthCallbackRoute = $derived(
        OAUTH_CALLBACK_ROUTES.some(route =>
            page.url.pathname.startsWith(route)
        ),
    );

    // Let's handle routing here, or at least call it in this layout function. Maybe move elsewhere and reference here
    // We can't put any changes in authState in the +page.ts files because they don't have access to the authState
    // Until after the routing has been determined so we might as well do it here I'd imagine
    // Can see if there's a better way of doing this. Open to other options.
    // This routing all sucks though and feels so weak. I'd rather not route someone to 2fa
    // They should be able to back out of this flow would be ideal. TBD. Maybe call a few functions here

    $effect(() => {
        const pathname = page.url.pathname;

        // These routes should all load the /login route unless user is authed with nonpending 2fa
        const isLoginRoute = LOGIN_ROUTES.includes(pathname);

        if (auth.userSessionToken.state === "loading") return;

        // Root redirect only can redirect to login or landing
        if (pathname === "/") {
            if (auth.userSessionToken.state === "authenticated") {
                goto(LANDING_ROUTE, { replaceState: true });
            } else {
                goto("/login", { replaceState: true });
            }
            return;
        }

        // Skip routing for oauth callbacks let them manage their own routing
        if (
            isOAuthCallbackRoute
            && auth.userSessionToken.state !== "authenticated"
        ) {
            return;
        }

        // Don't redirect 2FA if accessed correctly
        if (
            auth.userSessionToken.state === "authenticated"
            && auth.hasPendingMfa
            && pathname === "/two-factor"
        ) {
            return;
        }

        // Otherwise redirect to login
        if (
            !isLoginRoute
            && auth.userSessionToken.state !== "authenticated"
        ) {
            // TODO: Add redirectTo logic later
            goto(`/login`);
        }

        // Public routes logic
        if (
            (isLoginRoute || isOAuthCallbackRoute)
            && auth.userSessionToken.state === "authenticated"
        ) {
            goto(LANDING_ROUTE);
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

<!-- TODO: Add protection to routes if not logged in -->
<!-- This should go on each route somewhere or here might be sufficient -->
<!-- TODO: Clean this up at some point -->
{#if auth.userSessionToken.state === "loading"}
    <div>Loading...</div>
{:else if auth.userSessionToken.state === "unauthenticated"}
    {#if isOAuthCallbackRoute}
        <div class="login-page-container">
            <LoginHeader />
            {@render children()}
            <LoginFooter />
        </div>
    {:else}
        <div class="login-page-container">
            <LoginHeader />
            <LoginPage />
            <LoginFooter />
        </div>
    {/if}
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
