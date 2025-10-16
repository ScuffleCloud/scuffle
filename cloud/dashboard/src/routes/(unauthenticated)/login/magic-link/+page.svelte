<!-- The url that the magic link will return if accessed by a returning user -->
<script>
    import {
        useCompleteMagicLink,
    } from "$features/login/authMutations";
    import { handleMagicLinkCallback } from "$features/login/utils";
    import { onMount } from "svelte";

    const completeMagicLink = useCompleteMagicLink();

    onMount(() => {
        handleMagicLinkCallback(completeMagicLink.mutate);
    });
</script>

<div>Processing magic link...</div>
