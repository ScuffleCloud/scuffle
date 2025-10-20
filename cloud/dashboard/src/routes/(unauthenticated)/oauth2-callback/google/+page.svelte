<script>
    import AuthCallbackForm from "$features/login/auth-callbacks/auth-callback-form.svelte";
    import { handleGoogleOAuthCallback } from "$features/login/auth-callbacks/utils";
    import { useCompleteGoogleLogin } from "$features/login/authMutations";
    import { onMount } from "svelte";

    const completeGoogleLogin = useCompleteGoogleLogin();

    onMount(() => {
        handleGoogleOAuthCallback(completeGoogleLogin.mutate);
    });
</script>

<AuthCallbackForm
    message="Processing google oauth link..."
    mutation={completeGoogleLogin}
/>
