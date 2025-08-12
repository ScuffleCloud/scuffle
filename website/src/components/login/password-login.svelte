<script lang="ts">
    import { authState, authAPI, clearError, type AuthResult } from '$lib/authState.svelte';
    import { goto } from '$app/navigation';
    import IconArrowDialogLink from '$lib/images/icon-arrow-dialog-link.svelte';
    import TurnstileOverlay from '$components/turnstile-overlay.svelte';
    import IconArrowLeft from '$lib/images/icon-arrow-left.svelte';

    let turnstileOverlayComponent: TurnstileOverlay | null = null;
    const getToken = async () => await turnstileOverlayComponent?.getToken();

    // This initial value should come from local storage if it exists
    let email = $state<string>('');
    let password = $state<string>('');
    let localLoading = $state<boolean>(false);

    async function handleSubmit(event: SubmitEvent): Promise<void> {
        event.preventDefault();
        if (localLoading) return;

        localLoading = true;
        const token = await getToken();
        if (email && password && token) {
            try {
                // const result: AuthResult = await authAPI.loginWithPassword(email, password);
                // if (!result.success) {
                //     console.error('Login failed:', result.error);
                // }
            } catch (error) {
                console.error('Login error:', error);
            } finally {
                localLoading = false;
            }
        }
    }

    function handleBack(): void {
        goto('/log-in');
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

    function handlePasswordInput(): void {
        if (authState.error) {
            clearError();
        }
    }

    const isLoading = $derived(authState.isLoading || localLoading);
</script>

<div class="login-card">
    <div class="header">
        <button type="button" onclick={handleBack} class="back-button">
            <IconArrowLeft />
        </button>
        <h1 class="title">Password Login</h1>
    </div>

    {#if authState.error}
        <div class="error-message">
            {authState.error}
        </div>
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
                placeholder="Enter your email"
                disabled={isLoading}
                required
                autocomplete="email"
            />
        </div>

        <div class="form-group">
            <label for="password" class="form-label">Password</label>
            <input
                type="password"
                id="password"
                bind:value={password}
                oninput={handlePasswordInput}
                class="form-input"
                placeholder="Enter your password"
                disabled={isLoading}
                required
                autocomplete="current-password"
            />
        </div>

        <button type="submit" class="btn-primary" disabled={isLoading}>
            {#if isLoading}
                <span class="loading-spinner-small"></span>
                Logging in...
            {:else}
                Continue
            {/if}
        </button>
    </form>
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

    .header {
        display: flex;
        align-items: center;
        position: relative;
        margin-bottom: 2rem;
    }

    .back-button {
        background: none;
        border: none;
        color: #6b7280;
        cursor: pointer;
        font-size: 0.875rem;
        padding: 0;
        position: absolute;
        left: 0;
        top: 50%;
        transform: translateY(-50%);
        display: flex;
        align-items: center;
        gap: 0.25rem;
    }

    .back-button:hover {
        color: #374151;
    }

    .title {
        font-size: 1.5rem;
        font-weight: 600;
        margin: 0 auto;
        text-align: center;
        flex: 1;
    }

    .error-message {
        background-color: #fef2f2;
        border: 1px solid #fecaca;
        color: #dc2626;
        padding: 0.75rem;
        border-radius: 0.5rem;
        margin-bottom: 1.5rem;
        font-size: 0.875rem;
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

    .footer-links {
        display: flex;
        justify-content: center;
        margin: 2rem 0 1.25rem 0;
        gap: 1rem;
        align-items: center;

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
