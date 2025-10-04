<script>
    import { useMagicLinkAuth } from "$lib/auth/magicLinkAuth.svelte";
    import { onMount } from "svelte";

    const { handleMagicLinkCallback } = useMagicLinkAuth();

    onMount(() => {
        handleMagicLinkCallback();
    });
</script>

<div>Processing magic link...</div>
