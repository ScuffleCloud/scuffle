<script lang="ts">
    import {
        DEFAULT_LOGIN_MODE,
        type LoginMode,
    } from "$components/streams/types";
    import TurnstileOverlay from "$components/turnstile-overlay.svelte";
    import { useGoogleAuth } from "$lib/auth/googleAuth.svelte";
    import { sessionsServiceClient } from "$lib/grpcClient";
    import IconArrowDialogLink from "$lib/images/icon-arrow-dialog-link.svelte";
    import { CaptchaProvider } from "@scufflecloud/proto/scufflecloud/core/v1/common.js";
    import ForgotPasswordForm from "./forgot-password-form.svelte";
    import MagicLinkForm from "./magic-link-form.svelte";
    import MagicLinkSent from "./magic-link-sent.svelte";
    import PasskeyForm from "./passkey-form.svelte";
    import PasswordForm from "./password-form.svelte";
    import PasswordResetSent from "./password-reset-sent.svelte";
    import SigninOptions from "./signin-options.svelte";

    let turnstileOverlayComponent: TurnstileOverlay | null = null;

    function getInitialLoginModeFromUrl(): LoginMode {
        const path = window.location.pathname;
        if (path.includes("/password")) return "password";
        if (path.includes("/forgot-password")) return "forgot-password";
        if (path.includes("/passkey")) return "passkey";
        return DEFAULT_LOGIN_MODE;
    }

    let navigationSource = $state<"direct" | "internal">("direct");
    let loginMode = $state<LoginMode>(getInitialLoginModeFromUrl());

    // Set we can set back button navigation correctly
    function changeLoginMode(mode: LoginMode) {
        navigationSource = "internal";
        loginMode = mode;
    }

    // Manage routing here. Will add shallow routing
    let isRestoringFromHistory = false;
    $effect(() => {
        function handlePopState(event: PopStateEvent) {
            isRestoringFromHistory = true;
            if (event.state?.loginMode) {
                loginMode = event.state.loginMode;
            } else {
                window.history.back();
            }
        }

        window.addEventListener("popstate", handlePopState);

        if (!isRestoringFromHistory) {
            history.pushState({ loginMode }, "", loginMode);
        }
        isRestoringFromHistory = false;

        return () => {
            window.removeEventListener("popstate", handlePopState);
        };
    });

    const getToken = async () =>
        await turnstileOverlayComponent?.getToken();

    const turnstileLoading = $derived(
        /* eslint-disable-next-line @typescript-eslint/no-explicit-any */
        (turnstileOverlayComponent as any)?.showTurnstileOverlay
            || false,
    );

    let userEmail = $state<string>("");
    let isLoading = $state<boolean>(false);

    // Let's move this to a common logic function eventually like google auth
    async function handleMagicLinkSubmit(email: string): Promise<void> {
        const token = await getToken();
        if (email && token) {
            try {
                const call = sessionsServiceClient.loginWithMagicLink({
                    captcha: {
                        provider: CaptchaProvider.TURNSTILE,
                        token: token,
                    },
                    email,
                });
                const status = await call.status;
                const response = await call.response;
                console.log("Magic link response:", response);
                // TODO: Verify implementation after email flow is finished
                if (status.code === "OK") {
                    userEmail = email;
                    loginMode = "magic-link-sent";
                } else {
                    console.error("Magic link failed:", status.detail);
                }
            } catch (error) {
                console.error("Magic link error:", error);
            }
        }
    }

    async function handlePasswordSubmit(
        email: string,
        password: string,
    ): Promise<void> {
        const token = await getToken();
        if (email && password && token) {
            try {
                // const result: AuthResult = await authAPI.loginWithPassword(email, password);
                console.log("Password login for:", email);
            } catch (error) {
                console.error("Password login error:", error);
            }
        }
    }

    async function handlePasskeySubmit(email: string): Promise<void> {
        const token = await getToken();
        if (email && token) {
            try {
                // const result: AuthResult = await authAPI.loginWithPasskey(email);
                console.log("Passkey login for:", email);
            } catch (error) {
                console.error("Passkey login error:", error);
            }
        }
    }

    async function handleForgotPasswordSubmit(
        email: string,
    ): Promise<void> {
        const token = await getToken();
        if (email && token) {
            try {
                // const result: AuthResult = await authAPI.sendPasswordReset(email);
                console.log("Password reset for:", email);
                userEmail = email;
                loginMode = "password-reset-sent";
            } catch (error) {
                console.error("Password reset error:", error);
            }
        }
    }

    function handleBack(backRoute?: LoginMode): void {
        if (navigationSource === "internal") {
            window.history.back();
        } else {
            loginMode = backRoute || "login";
        }
    }

    const googleAuth = useGoogleAuth();

    // If googleAuth is loading, show a loading spinner
    // Combine with other
    $effect(() => {
        if (googleAuth.loading()) {
            isLoading = true;
        } else {
            isLoading = false;
        }
    });
</script>

<div class="login-card">
    {#if loginMode === "login"}
        <MagicLinkForm onSubmit={handleMagicLinkSubmit} {isLoading} />
        <SigninOptions
            {googleAuth}
            onModeChange={changeLoginMode}
            {isLoading}
        />
    {:else if loginMode === "password"}
        <PasswordForm
            onSubmit={handlePasswordSubmit}
            onBack={() => handleBack()}
            isLoading={isLoading || turnstileLoading}
        />
    {:else if loginMode === "passkey"}
        <PasskeyForm
            onSubmit={handlePasskeySubmit}
            onBack={() => handleBack()}
            {isLoading}
        />
    {:else if loginMode === "magic-link-sent"}
        <MagicLinkSent email={userEmail} />
    {:else if loginMode === "forgot-password"}
        <ForgotPasswordForm
            onSubmit={handleForgotPasswordSubmit}
            onBack={() => handleBack("password")}
            {isLoading}
        />
    {:else if loginMode === "password-reset-sent"}
        <PasswordResetSent
            email={userEmail}
            onBack={() => handleBack("password")}
        />
    {/if}
</div>

<div class="footer-links">
    {#if loginMode === "login"}
        <a
            href="/password"
            onclick={(e) => {
                e.preventDefault();
                changeLoginMode("password");
            }}
            class="link"
            class:disabled={isLoading}
        >
            Login with password
        </a>
        <a
            href="/contact-support"
            class="link"
            class:disabled={isLoading}
        >
            Contact Support <IconArrowDialogLink />
        </a>
    {:else if loginMode === "password"}
        <a
            href="/forgot-password"
            onclick={(e) => {
                e.preventDefault();
                changeLoginMode("forgot-password");
            }}
            class="link"
            class:disabled={isLoading}
        >
            Forgot Password
        </a>
        <a
            href="/contact-support"
            class="link"
            class:disabled={isLoading}
        >
            Contact Support <IconArrowDialogLink />
        </a>
    {:else if loginMode === "passkey"}
        <a
            href="/contact-support"
            class="link"
            class:disabled={isLoading}
        >
            Contact Support <IconArrowDialogLink />
        </a>
    {/if}
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

    .footer-links {
      display: flex;
      justify-content: space-between;
      margin: 2rem 0 1.25rem 0;
      gap: 1rem;
      align-items: center;
    }

    a {
      background: none;
      border: none;
      color: #6b7280;
      cursor: pointer;
      text-decoration: none;
      display: flex;
      align-items: center;
      gap: 0.25rem;
      font-size: 0.875rem;
    }

    a:hover:not(.disabled) {
      color: #374151;
      text-decoration: underline;
    }

    .link.disabled {
      color: #9ca3af;
      cursor: not-allowed;
      pointer-events: none;
    }

    @media (max-width: 480px) {
      .login-card {
        padding: 1.5rem;
        margin: 0 1rem;
      }

      .footer-links {
        margin: 1rem 0 1rem 0;
      }
    }
</style>
