<script>
    import { useAuth } from "$lib/auth.svelte";
    import { onMount } from "svelte";

    const auth = useAuth();

    onMount(async () => {
        await auth.logout();
    });
</script>
