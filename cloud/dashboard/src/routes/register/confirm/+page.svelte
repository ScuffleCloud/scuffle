<!-- The url that the magic link will return if accessed by a new user -->
<script>
    import { onMount } from "svelte";
    import {
        useCompleteMagicLink,
    } from "../../../features/login/authMutations";
    import { handleMagicLinkCallback } from "../../../features/login/utils";

    const completeMagicLink = useCompleteMagicLink();

    onMount(() => {
        handleMagicLinkCallback(completeMagicLink.mutate);
    });
</script>

<div>Processing sign up...</div>
