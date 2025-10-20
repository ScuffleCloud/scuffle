<script>
    import AuthCallbackForm from "$features/login/auth-callbacks/auth-callback-form.svelte";
    import { handleGoogleOAuthCallback } from "$features/login/auth-callbacks/utils";
    import { onMount } from "svelte";
    import { useCompleteGoogleLogin } from "../../../../features/login/authMutations";

    const completeGoogleLogin = useCompleteGoogleLogin();

    onMount(() => {
        handleGoogleOAuthCallback(completeGoogleLogin.mutate);
    });
</script>

<AuthCallbackForm
    message="Processing google oauth link..."
    mutation={completeGoogleLogin}
/>
