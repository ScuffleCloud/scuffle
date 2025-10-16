<script>
    import { onMount } from "svelte";
    import { useCompleteGoogleLogin } from "../../../../features/login/authMutations";

    const completeGoogleLogin = useCompleteGoogleLogin();

    onMount(() => {
        const urlParams = new URLSearchParams(window.location.search);
        const code = urlParams.get("code");
        const state = urlParams.get("state");

        if (code && state) {
            completeGoogleLogin.mutate({ code, state });
        }
    });
</script>

<div>Processing google oauth link...</div>
