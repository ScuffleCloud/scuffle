<!-- The url that the magic link will return if accessed by a new user -->
<script>
    import { useMagicLinkAuth } from "$lib/auth/magicLinkAuth.svelte";
    import { onMount } from "svelte";

    const { handleMagicLinkCallback } = useMagicLinkAuth();

    onMount(() => {
        handleMagicLinkCallback();
    });
</script>

<div>Processing sign up...</div>
