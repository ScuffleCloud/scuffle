<!-- The url that the magic link will return if accessed by a new user -->
<script>
    import AuthCallbackForm from "$features/login/auth-callbacks/auth-callback-form.svelte";
    import { handleMagicLinkCallback } from "$features/login/auth-callbacks/utils";
    import {
        useCompleteMagicLink,
    } from "$features/login/authMutations";
    import { onMount } from "svelte";

    const completeMagicLink = useCompleteMagicLink();

    onMount(() => {
        handleMagicLinkCallback(completeMagicLink.mutate);
    });
</script>

<AuthCallbackForm
    message="Processing sign up..."
    mutation={completeMagicLink}
/>
