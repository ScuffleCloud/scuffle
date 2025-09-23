<script>
    import { authState } from "$lib/auth.svelte";
    import { onMount } from "svelte";

    onMount(async () => {
        await authState().logout();
    });
</script>
