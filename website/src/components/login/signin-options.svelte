<script>
    import IconGoogle from '$lib/images/icon-google.svelte';
    import IconLoginKey from '$lib/images/icon-login-key.svelte';

    let isLoading = $state(false);

    function handleGoogleLogin() {
        console.log('Google login');
    }

    function handlePasskeyLogin() {
        console.log('Passkey login');
    }
</script>

<div class="divider">OR</div>
<button type="button" onclick={handleGoogleLogin} class="btn-social google" disabled={isLoading}>
    <IconGoogle />
    Continue with Google
</button>

<button type="button" onclick={handlePasskeyLogin} class="btn-social passkey" disabled={isLoading}>
    <IconLoginKey />
    Continue with Passkey
</button>

<style>
    .divider {
        display: flex;
        align-items: center;
        margin: 2rem 0;
        color: #9ca3af;
        font-size: 0.875rem;
        text-transform: uppercase;
    }

    .divider::before,
    .divider::after {
        content: '';
        flex: 1;
        height: 1px;
        background: #d1d5db;
    }

    .divider::before {
        margin-right: 0.325rem;
    }

    .divider::after {
        margin-left: 0.325rem;
    }

    .btn-social {
        width: 100%;
        padding: 0.75rem;
        background: white;
        color: #374151;
        border: 1px solid #d1d5db;
        cursor: pointer;
        margin-bottom: 0.5rem;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        border-radius: 0.5rem;
    }

    .btn-social:hover:not(:disabled) {
        background: #f9fafb;
        border-color: #9ca3af;
    }

    .btn-social:disabled {
        background: white;
        color: #9ca3af;
        cursor: not-allowed;
    }
</style>
