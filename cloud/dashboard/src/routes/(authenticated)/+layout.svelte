<script lang="ts">
    import { browser } from "$app/environment";
    import Navbar from "$components/left-nav/navbar.svelte";
    import RightNav from "$components/right-nav/right-nav.svelte";
    import TopNav from "$components/top-nav/top-nav.svelte";
    import {
        QueryClient,
        QueryClientProvider,
    } from "@tanstack/svelte-query";

    const { children } = $props();

    const queryClient = new QueryClient({
        defaultOptions: {
            queries: {
                enabled: browser,
            },
        },
    });
</script>

<!-- Authenticated -->
<QueryClientProvider client={queryClient}>
    <div class="app">
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
    </div>
</QueryClientProvider>

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
