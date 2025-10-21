<script lang="ts">
    import LoginCard from "$features/login/login-card.svelte";
    import { authState } from "$lib/auth.svelte";
    import LoadingSpinner from "$lib/components/loading-spinner.svelte";
    import IconArrowDialogLink from "$lib/images/icon-arrow-dialog-link.svelte";
    import { MfaOption } from "@scufflecloud/proto/scufflecloud/core/v1/sessions_service.js";
    import RecoveryCodeForm from "./recovery-code-form.svelte";
    import TotpForm from "./totp-form.svelte";
    import {
        DEFAULT_TWO_FACTOR_MODE,
        type TwoFactorMode,
    } from "./types";
    import WebAuthnnForm from "./web-authnn-form.svelte";

    const auth = authState();

    // Null means we are in a loading state as page cannot be accessed if there are no mfa options
    // Can change how this looks down the line if we want
    const mfaOptions = $derived(
        auth.userSessionToken.state === "authenticated"
            ? (auth.userSessionToken.data.mfaPending ?? null)
            : null,
    );

    const isLoading = $derived(mfaOptions === null);

    // The URL's are not updated by design
    function getInitialTwoFactorModeFromState(): TwoFactorMode {
        // Prioritize webauthn over totp here
        if (mfaOptions?.includes(MfaOption.WEB_AUTHN)) {
            return "webauthn";
        }
        if (mfaOptions?.includes(MfaOption.TOTP)) return "totp";
        if (mfaOptions?.includes(MfaOption.RECOVERY_CODES)) {
            return "recovery-code";
        }

        return DEFAULT_TWO_FACTOR_MODE;
    }

    // Use SvelteKit's page state instead of local state
    let twoFactorMode = $state(getInitialTwoFactorModeFromState());
    let previousTwoFactorMode = $state(
        getInitialTwoFactorModeFromState(),
    );

    function changeTwoFactorMode(mode: TwoFactorMode) {
        previousTwoFactorMode = twoFactorMode;
        twoFactorMode = mode;
    }
</script>

<svelte:head>
    <title>Scuffle | Login</title>
</svelte:head>

<LoginCard>
    {#if isLoading}
        <LoadingSpinner />
    {:else}
        {#if twoFactorMode === "webauthn"}
            <WebAuthnnForm
                onToptModeChange={mfaOptions?.includes(MfaOption.TOTP)
                ? (() => changeTwoFactorMode("totp"))
                : null}
                onBackupCodeChange={() => changeTwoFactorMode("recovery-code")}
            />
        {:else if twoFactorMode === "totp"}
            <TotpForm
                onModeChange={mfaOptions?.includes(MfaOption.WEB_AUTHN)
                ? (() => changeTwoFactorMode("webauthn"))
                : null}
                onBackupCodeChange={() => changeTwoFactorMode("recovery-code")}
            />
        {:else if twoFactorMode === "recovery-code"}
            <RecoveryCodeForm
                onBack={() => changeTwoFactorMode(previousTwoFactorMode)}
            />
        {/if}
    {/if}
</LoginCard>
<div class="footer-links">
    <a
        href="/contact-support"
        class="link"
        class:disabled={isLoading}
    >
        Contact Support <IconArrowDialogLink />
    </a>
</div>

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
      .footer-links {
        margin: 1rem 0 1rem 0;
      }
    }
</style>
