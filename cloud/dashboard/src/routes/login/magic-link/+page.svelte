<!-- The url that the magic link will return if accessed by a returning user -->
<script>
    import { onMount } from "svelte";
    import { useMagicLinkAuth } from "../../../features/login/magicLinkAuth.svelte";

    const { handleMagicLinkCallback } = useMagicLinkAuth();

    onMount(() => {
        handleMagicLinkCallback();
    });
</script>

<div>Processing magic link...</div>
