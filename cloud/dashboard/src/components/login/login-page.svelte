<script lang="ts">
    import { pushState } from "$app/navigation";
    import { page } from "$app/state";
    import {
        DEFAULT_LOGIN_MODE,
        type LoginMode,
    } from "$components/streams/types";
    import TurnstileOverlay from "$components/turnstile-overlay.svelte";
    import { useGoogleAuth } from "$lib/auth/googleAuth.svelte";
    import { useMagicLinkAuth } from "$lib/auth/magicLinkAuth.svelte";
    import IconArrowDialogLink from "$lib/images/icon-arrow-dialog-link.svelte";
    import { createSmartBack } from "$lib/navigation.svelte";
    import ForgotPasswordForm from "./forgot-password-form.svelte";
    import LoginCard from "./login-card.svelte";
    import MagicLinkForm from "./magic-link-form.svelte";
    import MagicLinkSent from "./magic-link-sent.svelte";
    import PasskeyForm from "./passkey-form.svelte";
    import PasswordForm from "./password-form.svelte";
    import PasswordResetSent from "./password-reset-sent.svelte";
    import SigninOptions from "./signin-options.svelte";

    let turnstileOverlayComponent: TurnstileOverlay | null = null;

    // If ex. magic-link page is shown it should be routed outside of this window to the correct path
    function getInitialLoginModeFromUrl(): LoginMode {
        const path = window.location.pathname;
        if (path.includes("/password")) return "password";
        if (path.includes("/forgot-password")) return "forgot-password";
        if (path.includes("/passkey")) return "passkey";
        return DEFAULT_LOGIN_MODE;
    }

    // Use SvelteKit's page state instead of local state
    let loginMode = $derived(
        page.state.loginMode ?? getInitialLoginModeFromUrl(),
    );

    // So back button navigation works as expected. Smart shallow routing.
    function changeLoginMode(mode: LoginMode) {
        const urlMap: Record<LoginMode, string> = {
            "login": "/",
            "password": "/password",
            "forgot-password": "/forgot-password",
            "passkey": "/passkey",
            "magic-link-sent": "/magic-link-sent",
            "password-reset-sent": "/password-reset-sent",
        };

        smartBack.markNavigation();

        pushState(urlMap[mode] || "/", { loginMode: mode });
    }

    const smartBack = createSmartBack(
        "login" as LoginMode,
        changeLoginMode,
    );

    const handleBack = smartBack.back;

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
        isLoading = true;
        const token = await getToken();
        if (!token) return;

        try {
            await magicLinkAuth.sendMagicLink(email, token);

            isLoading = false;
            // Magic link has successfully been sent
            userEmail = email;
            pushState("/magic-link-sent", {
                loginMode: "magic-link-sent",
                userEmail: email,
            });
        } catch (error) {
            console.error("Magic link error:", error);
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
                pushState("/password-reset-sent", {
                    loginMode: "password-reset-sent",
                    userEmail: email,
                });
            } catch (error) {
                console.error("Password reset error:", error);
            }
        }
    }

    const googleAuth = useGoogleAuth();
    const magicLinkAuth = useMagicLinkAuth();

    $effect(() => {
        if (googleAuth.loading()) {
            isLoading = true;
        } else {
            isLoading = false;
        }
    });

    // Handle OAuth callbacks
    $effect(() => {
        googleAuth.handleOAuthCallback();
        magicLinkAuth.handleMagicLinkCallback();
    });

    // See if I need this later
    $effect(() => {
        if (page.state.userEmail) {
            userEmail = page.state.userEmail;
        }
    });
</script>

<svelte:head>
    <title>Scuffle | Login</title>
</svelte:head>

<LoginCard>
    {#if loginMode === "login"}
        <MagicLinkForm onSubmit={handleMagicLinkSubmit} {isLoading} />
        <SigninOptions
            onSubmit={googleAuth.initiateLogin}
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
</LoginCard>

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
    .footer-links {
      display: flex;
      justify-content: space-between;
      margin-bottom: 1.25rem;
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
