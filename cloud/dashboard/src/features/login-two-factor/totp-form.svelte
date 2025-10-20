<script lang="ts">
    import LoginFormTitle from "$features/login/login-form-title.svelte";
    import CodeInput from "$lib/components/code-input.svelte";
    import LoginOrDivider from "$lib/components/login-or-divider.svelte";
    import IconLoginKey from "$lib/images/icon-login-key.svelte";
    import { useValidateMfaTotp } from "./mfaChallengeMutations";
    import RecoveryCodeCollapsible from "./recovery-code-collapsible.svelte";

    interface Props {
        onModeChange: (() => void) | null;
        onBackupCodeChange: () => void;
    }

    let { onModeChange, onBackupCodeChange }: Props = $props();

    const validateMfaTotpMutation = useValidateMfaTotp();

    let pinValue = $state("");

    async function handleContinue() {
        if (pinValue.length === 6) {
            validateMfaTotpMutation.mutate(pinValue);
        }
    }
    let continueRef = $state<HTMLButtonElement | null>(null);

    const onBack = $derived(() => {
        if (onModeChange) {
            onModeChange();
        }
    });
</script>

<LoginFormTitle title="MFA Login" {onBack} />
<p class="subtitle">
    Enter the 6-digit code from your 2FA authenticator app below
</p>

<CodeInput
    bind:value={pinValue}
    disabled={validateMfaTotpMutation.isPending || validateMfaTotpMutation.isSuccess}
    maxLength={6}
    type="numeric"
    placeholder="-"
/>

<button
    bind:this={continueRef}
    type="button"
    onclick={handleContinue}
    class="continue-btn"
    disabled={validateMfaTotpMutation.isPending || validateMfaTotpMutation.isSuccess
    || pinValue.length !== 6}
>
    {#if validateMfaTotpMutation.isPending}
        <div class="spinner"></div>
        Verifying...
    {:else}
        Continue
    {/if}
</button>

{#if onModeChange}
    <LoginOrDivider />
    <button
        type="button"
        onclick={() => onModeChange()}
        class="continue-btn"
    >
        <IconLoginKey />
        Continue with Passkey
    </button>
{/if}

<RecoveryCodeCollapsible onAction={onBackupCodeChange} />

<style>
    .subtitle {
      color: #6b7280;
      font-size: 0.95rem;
      line-height: 1.5;
      margin: 0 0 2rem 0;
    }

    .continue-btn {
      width: 100%;
      padding: 0.875rem;
      background: var(--colors-yellow40);
      color: var(--colors-yellow80);
      border: none;
      border-radius: 0.5rem;
      font-size: 1rem;
      font-weight: 600;
      cursor: pointer;
      transition: background-color 0.2s;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
    }

    .continue-btn:hover:not(:disabled) {
      background: #d97706;
    }

    .continue-btn:disabled {
      background: #d1d5db;
      cursor: not-allowed;
    }

    .spinner {
      width: 16px;
      height: 16px;
      border: 2px solid rgba(255, 255, 255, 0.3);
      border-top: 2px solid white;
      border-radius: 50%;
      animation: spin 1s linear infinite;
    }

    @keyframes spin {
      0% {
        transform: rotate(0deg);
      }
      100% {
        transform: rotate(360deg);
      }
    }

    @media (max-width: 480px) {
      .mfa-container {
        padding: 1.5rem;
        margin: 1rem;
      }
    }
</style>
