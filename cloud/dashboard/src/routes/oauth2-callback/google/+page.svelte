<script>
    import { useGoogleAuth } from "$lib/auth/googleAuth.svelte";
    import { onMount } from "svelte";

    const { handleOAuthCallback } = useGoogleAuth();

    onMount(() => {
        handleOAuthCallback();
    });
</script>

<div>Processing google oauth link...</div>
