<script lang="ts">
    import { authState, authAPI, clearError, type AuthResult } from '$lib/authState.svelte';
    import IconArrowDialogLink from '$lib/images/icon-arrow-dialog-link.svelte';
    import SigninOptions from './signin-options.svelte';
    import TurnstileOverlay from '$components/turnstile-overlay.svelte';

    let turnstileOverlayComponent: TurnstileOverlay | null = null;
    const getToken = async () => await turnstileOverlayComponent?.getToken();

    let email = $state<string>('tim.jennings@example.com');
    let localLoading = $state<boolean>(false);

    // Get this from authState probably TBD
    let magicLinkSent = $state<boolean>(false);

    async function handleSubmit(event: SubmitEvent): Promise<void> {
        event.preventDefault();
        if (localLoading) return;

        localLoading = true;
        const token = await getToken();
        if (email && token) {
            // Hopefully this authAPI comes with ways to check query status instead of using local state
            try {
                const result: AuthResult = await authAPI.sendMagicLink(email);
                if (!result.success) {
                    console.error('Login failed:', result.error);
                } else {
                    magicLinkSent = true;
                }
                console.log('magicLinkSent', magicLinkSent);
            } catch (error) {
                console.error('Login error:', error);
            } finally {
                localLoading = false;
            }
        }
    }

    function handleContactSupport(): void {
        console.log('Contact support clicked');
    }

    // Clear errors when user starts typing
    function handleEmailInput(): void {
        if (authState.error) {
            clearError();
        }
    }

    const isLoading = $derived(authState.isLoading || localLoading);
</script>

<div class="login-card">
    {#if !magicLinkSent}
        <h1 class="title">Log in to Scuffle</h1>
        {#if authState.error}
            {authState.error}
        {/if}
        <form onsubmit={handleSubmit} class="login-form">
            <div class="form-group">
                <label for="email" class="form-label">Email</label>
                <input
                    type="email"
                    id="email"
                    bind:value={email}
                    oninput={handleEmailInput}
                    class="form-input"
                    placeholder="tim.jennings@example.com"
                    disabled={isLoading}
                    required
                    autocomplete="email"
                />
            </div>

            <button type="submit" class="btn-primary" disabled={isLoading || !email.trim()}>
                {#if isLoading}
                    <span class="loading-spinner-small"></span>
                    Logging in...
                {:else}
                    Continue with email
                {/if}
            </button>
        </form>
        <SigninOptions />
    {:else}
        <h1 class="title">Check your email for a magic link to continue!</h1>
        <p class="subtitle">We've sent you an email with a magic link to verify your account.</p>
    {/if}
</div>
<div class="footer-links">
    <button type="button" onclick={handleContactSupport} class="link" disabled={isLoading}>
        Contact Support <IconArrowDialogLink />
    </button>
</div>
<TurnstileOverlay bind:this={turnstileOverlayComponent} />

<style>
    .login-card {
        border-radius: 1.25rem;
        padding: 2.75rem;
        width: 100%;
        max-width: 400px;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05);
        border: 1px solid var(--colors-gray50);
        background-color: var(--colors-gray20);
        text-align: center;
    }

    .title {
        font-size: 1.5rem;
        font-weight: 600;
        margin: 0 0 2rem 0;
    }

    .login-form {
        margin-bottom: 0.5rem;
    }

    .form-group {
        margin-bottom: 1.25rem;
        text-align: left;
    }

    .form-label {
        display: block;
        font-size: 0.875rem;
        font-weight: 500;
        color: #374151;
        margin-bottom: 0.375rem;
    }

    .form-input {
        width: 100%;
        padding: 0.75rem 1rem;
        border: 1px solid #d1d5db;
        border-radius: 0.5rem;
        font-size: 1rem;
        background: white;
        box-sizing: border-box;
        transition:
            border-color 0.2s,
            box-shadow 0.2s;
    }

    .form-input:focus {
        outline: none;
        border-color: #f59e0b;
        box-shadow: 0 0 0 3px rgba(245, 158, 11, 0.1);
    }

    .form-input:disabled {
        background: #f9fafb;
        color: #9ca3af;
        cursor: not-allowed;
    }

    .btn-primary {
        width: 100%;
        padding: 0.75rem;
        background: var(--colors-yellow40);
        color: var(--colors-yellow80);
        border: none;
        border-radius: 0.5rem;
        font-size: 1rem;
        font-weight: 500;
        cursor: pointer;
        transition: background-color 0.2s;
        margin-bottom: 0.5rem;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
    }

    .btn-primary:hover:not(:disabled) {
        background: #d97706;
    }

    .btn-primary:disabled {
        background: #9ca3af;
        cursor: not-allowed;
    }

    .footer-links {
        display: flex;
        justify-content: space-between;
        margin: 2rem 0 1.25rem 0;
        gap: 1rem;
        align-items: center;
    }

    .link {
        background: none;
        border: none;
        color: #6b7280;
        cursor: pointer;
        text-decoration: none;
        display: flex;
        align-items: center;
        gap: 0.25rem;
    }

    .link:hover:not(:disabled) {
        color: #374151;
        text-decoration: underline;
    }

    .link:disabled {
        color: #9ca3af;
        cursor: not-allowed;
    }

    .subtitle {
        font-size: 0.875rem;
        color: #6b7280;
        margin-bottom: 1.25rem;
    }

    @media (max-width: 480px) {
        .login-card {
            padding: 1.5rem;
            margin: 0 1rem;
        }

        .footer-links {
            flex-direction: column;
            gap: 0.5rem;
        }
    }
</style>
