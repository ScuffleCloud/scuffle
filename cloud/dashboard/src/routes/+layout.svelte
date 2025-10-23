<script lang="ts">
    import "$styles/global.css";
    import "$styles/variables.css";
    import "@fontsource-variable/archivo";
    import { browser } from "$app/environment";
    import { authState } from "$lib/auth.svelte";
    import {
        QueryClient,
        QueryClientProvider,
    } from "@tanstack/svelte-query";

    const auth = authState();

    const { children } = $props();

    const queryClient = new QueryClient({
        defaultOptions: {
            queries: {
                enabled: browser,
                staleTime: 1000 * 60 * 5,
            },
        },
    });
</script>

<QueryClientProvider client={queryClient}>
    {#if auth.userSessionToken.state === "loading"}
        <div>Loading...</div>
    {:else}
        {@render children()}
    {/if}
</QueryClientProvider>
