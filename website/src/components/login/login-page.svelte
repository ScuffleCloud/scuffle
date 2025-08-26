<script lang="ts">
import { DEFAULT_LOGIN_MODE, type LoginMode } from "$components/streams/types";
import TurnstileOverlay from "$components/turnstile-overlay.svelte";
import {
  authAPI,
  type AuthResult,
  authState,
  clearError,
} from "$lib/authState.svelte";
import IconArrowDialogLink from "$lib/images/icon-arrow-dialog-link.svelte";
import ForgotPasswordForm from "./forgot-password-form.svelte";
import MagicLinkForm from "./magic-link-form.svelte";
import MagicLinkSent from "./magic-link-sent.svelte";
import PasskeyForm from "./passkey-form.svelte";
import PasswordForm from "./password-form.svelte";
import PasswordResetSent from "./password-reset-sent.svelte";
import SigninOptions from "./signin-options.svelte";

let turnstileOverlayComponent: TurnstileOverlay | null = null;
let loginMode = $state<LoginMode>(DEFAULT_LOGIN_MODE);

// Manage routing here. Could add shallow routing in the future if we feel necessary
let isRestoringFromHistory = false;
$effect(() => {
  function handlePopState(event: PopStateEvent) {
    isRestoringFromHistory = true;
    if (event.state?.loginMode) {
      loginMode = event.state.loginMode;
    } else {
      loginMode = DEFAULT_LOGIN_MODE;
    }
  }

  window.addEventListener("popstate", handlePopState);

  if (!isRestoringFromHistory) {
    history.pushState({ loginMode }, "", window.location.href);
  }
  isRestoringFromHistory = false;

  return () => {
    window.removeEventListener("popstate", handlePopState);
  };
});

const getToken = async () => await turnstileOverlayComponent?.getToken();

let userEmail = $state<string>("");
let localLoading = $state<boolean>(false);

async function handleMagicLinkSubmit(email: string): Promise<void> {
  const token = await getToken();
  if (email && token) {
    try {
      const result: AuthResult = await authAPI.sendMagicLink(email);
      if (result.success) {
        userEmail = email;
        loginMode = "magic-link-sent";
      } else {
        console.error("Magic link failed:", result.error);
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

async function handleForgotPasswordSubmit(email: string): Promise<void> {
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

function handleBack(): void {
  loginMode = "magic-link";
}

// Clear errors when user starts typing
function handleEmailInput(): void {
  if (authState.error) {
    clearError();
  }
}

const isLoading = $derived(authState.isLoading || localLoading);

function handleContactSupport(): void {
  console.log("Contact support clicked");
}
</script>

<div class="login-card">
	{#if loginMode === "magic-link"}
		<MagicLinkForm onSubmit={handleMagicLinkSubmit} {isLoading} />
		<SigninOptions onModeChange={(mode) => (loginMode = mode)} {isLoading} />
	{:else if loginMode === "password"}
		<PasswordForm
			onSubmit={handlePasswordSubmit}
			onBack={handleBack}
			{isLoading}
		/>
	{:else if loginMode === "passkey"}
		<PasskeyForm
			onSubmit={handlePasskeySubmit}
			onBack={handleBack}
			{isLoading}
		/>
	{:else if loginMode === "magic-link-sent"}
		<MagicLinkSent email={userEmail} />
	{:else if loginMode === "forgot-password"}
		<ForgotPasswordForm
			onSubmit={handleForgotPasswordSubmit}
			onBack={handleBack}
			{isLoading}
		/>
	{:else if loginMode === "password-reset-sent"}
		<PasswordResetSent email={userEmail} onBack={handleBack} />
	{/if}
</div>

<div class="footer-links">
	{#if loginMode === "magic-link"}
		<button
			type="button"
			onclick={() => (loginMode = "password")}
			class="link"
			disabled={isLoading}
		>
			Login with password
		</button>
		<button
			type="button"
			onclick={handleContactSupport}
			class="link"
			disabled={isLoading}
		>
			Contact Support <IconArrowDialogLink />
		</button>
	{:else if loginMode === "password"}
		<button
			type="button"
			onclick={() => (loginMode = "forgot-password")}
			class="link"
			disabled={isLoading}
		>
			Forgot password?
		</button>
		<button
			type="button"
			onclick={handleContactSupport}
			class="link"
			disabled={isLoading}
		>
			Contact Support <IconArrowDialogLink />
		</button>
	{:else if loginMode === "passkey"}
		<button
			type="button"
			onclick={handleContactSupport}
			class="link"
			disabled={isLoading}
		>
			Contact Support <IconArrowDialogLink />
		</button>
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

.link {
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

.link:hover:not(:disabled) {
  color: #374151;
  text-decoration: underline;
}

.link:disabled {
  color: #9ca3af;
  cursor: not-allowed;
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
