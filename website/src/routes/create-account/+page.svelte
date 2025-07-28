<!-- src/routes/LoginPage.svelte -->
<script lang="ts">
    import { authState, authAPI, clearError, type AuthResult } from '$lib/authState.svelte';
    import IconGoogle from '$lib/images/icon-google.svelte';
    import IconLoginKey from '$lib/images/icon-login-key.svelte';
    import ScuffleLogo from '$lib/images/scuffle-logo.svelte';

    // Form handling
    let email = $state<string>('tim.jennings@example.com');
    let localLoading = $state<boolean>(false);

    async function handleSubmit(event: SubmitEvent): Promise<void> {
        event.preventDefault();
        if (email && !localLoading && !authState.isLoading) {
            localLoading = true;
            try {
                const result: AuthResult = await authAPI.loginWithEmail(email);
                if (!result.success) {
                    console.error('Login failed:', result.error);
                }
            } catch (error) {
                console.error('Login error:', error);
            } finally {
                localLoading = false;
            }
        }
    }

    async function handleGoogleLogin(): Promise<void> {
        if (!localLoading && !authState.isLoading) {
            localLoading = true;
            try {
                const result: AuthResult = await authAPI.loginWithGoogle();
                if (!result.success) {
                    console.error('Google login failed:', result.error);
                }
            } catch (error) {
                console.error('Google login error:', error);
            } finally {
                localLoading = false;
            }
        }
    }

    async function handlePasskeyLogin(): Promise<void> {
        if (!localLoading && !authState.isLoading) {
            localLoading = true;
            try {
                const result: AuthResult = await authAPI.loginWithPasskey();
                if (!result.success) {
                    console.error('Passkey login failed:', result.error);
                }
            } catch (error) {
                console.error('Passkey login error:', error);
            } finally {
                localLoading = false;
            }
        }
    }

    async function handleSignUp(): Promise<void> {
        if (email && !localLoading && !authState.isLoading) {
            localLoading = true;
            try {
                const result: AuthResult = await authAPI.signUp(email);
                if (result.success && result.message) {
                    alert(result.message);
                } else if (!result.success) {
                    console.error('Sign up failed:', result.error);
                }
            } catch (error) {
                console.error('Sign up error:', error);
            } finally {
                localLoading = false;
            }
        }
    }

    async function handleForgotPassword(): Promise<void> {
        if (email && !localLoading && !authState.isLoading) {
            localLoading = true;
            try {
                const result: AuthResult = await authAPI.forgotPassword(email);
                if (result.success && result.message) {
                    alert(result.message);
                } else if (!result.success) {
                    console.error('Forgot password failed:', result.error);
                }
            } catch (error) {
                console.error('Forgot password error:', error);
            } finally {
                localLoading = false;
            }
        }
    }

    function handleContactSupport(): void {
        console.log('Contact support clicked');
        // Your support contact logic here
        // Could open a modal, navigate to support page, etc.
    }

    // Clear errors when user starts typing
    function handleEmailInput(): void {
        if (authState.error) {
            clearError();
        }
    }

    function handleClearError(): void {
        clearError();
    }

    const isLoading = $derived(authState.isLoading || localLoading);
</script>

<div class="login-container">
    <div class="logo-container">
        <div class="logo-container-image">
            <ScuffleLogo />
        </div>
        scuffle
    </div>
    <div class="login-card">
        <h1 class="title">Create an account</h1>
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
                    Log in
                {/if}
            </button>
        </form>
        <button
            type="button"
            onclick={handleSignUp}
            class="btn-secondary"
            disabled={isLoading || !email.trim()}
        >
            Sign up
        </button>

        <div class="divider">OR</div>
        <button
            type="button"
            onclick={handleGoogleLogin}
            class="btn-social google"
            disabled={isLoading}
        >
            <IconGoogle />
            Continue with Google
        </button>

        <button
            type="button"
            onclick={handlePasskeyLogin}
            class="btn-social passkey"
            disabled={isLoading}
        >
            <IconLoginKey />
            Continue with Passkey
        </button>
    </div>
    <div class="footer-links">
        <button
            type="button"
            onclick={handleForgotPassword}
            class="link"
            disabled={isLoading || !email.trim()}
        >
            Forgot password?
        </button>
        <button type="button" onclick={handleContactSupport} class="link" disabled={isLoading}>
            Contact Support ↗
        </button>
    </div>

    <!-- Terms -->
    <div class="terms">
        <p>
            By logging in, you agree to our
            <a href="/privacy" class="terms-link">Privacy Policy</a>
            and
            <a href="/terms" class="terms-link">Terms of Use</a>
        </p>
    </div>
</div>

<style>
    .login-container {
        min-height: 100vh;
        display: flex;
        align-items: center;
        justify-content: center;
        background-color: #f5f3f0;
        padding: 2rem;
        flex-direction: column;
    }

    .login-card {
        background: white;
        border-radius: 0.75rem;
        padding: 2.75rem;
        width: 100%;
        max-width: 400px;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05);
        text-align: center;
    }

    .logo-container {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        color: var(--brown-800);
        font-size: 1.5rem;
        font-weight: 800;
        text-transform: uppercase;
        text-decoration: none;
        margin-bottom: 2rem;

        .logo-container-image {
            display: flex;
            align-items: center;
            justify-content: center;
            filter: drop-shadow(0px 2px 4px 0px rgb(0, 0, 0, 0.05));
        }
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

    .btn-secondary {
        width: 100%;
        padding: 0.75rem;
        background: #f9fafb;
        color: #374151;
        border: 1px solid #d1d5db;
        border-radius: 0.5rem;
        font-size: 1rem;
        font-weight: 500;
        cursor: pointer;
    }

    .btn-secondary:hover:not(:disabled) {
        background: #f3f4f6;
        border-color: #9ca3af;
    }

    .btn-secondary:disabled {
        background: #f9fafb;
        color: #9ca3af;
        cursor: not-allowed;
    }

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

    .footer-links {
        display: flex;
        justify-content: space-between;
        margin: 2rem 0 1.25rem 0;
        gap: 1rem;
    }

    .link {
        background: none;
        border: none;
        color: #6b7280;
        cursor: pointer;
        text-decoration: none;
    }

    .link:hover:not(:disabled) {
        color: #374151;
        text-decoration: underline;
    }

    .link:disabled {
        color: #9ca3af;
        cursor: not-allowed;
    }

    .terms {
        font-size: 12px;
        color: #6b7280;
        line-height: 1.5;
    }

    .terms p {
        margin: 0;
    }

    .terms-link {
        color: #6b7280;
        text-decoration: underline;
    }

    .terms-link:hover {
        color: #374151;
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
