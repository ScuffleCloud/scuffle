<script lang="ts">
    import "$styles/global.css";
    import "$styles/variables.css";
    import "@fontsource-variable/archivo";
    import { authState } from "$lib/auth.svelte";

    const auth = authState();

    const { children } = $props();
</script>

{#if auth.userSessionToken.state === "loading"}
    <div>Loading...</div>
{:else}
    {@render children()}
{/if}
