<script lang="ts">
    import LoginFormTitle from "$features/login/login-form-title.svelte";
    import InlineNotification from "$lib/components/inline-notification.svelte";
    import { useValidateMfaRecoveryCode } from "./mfaChallengeMutations";

    interface Props {
        onBack: () => void;
    }

    let { onBack }: Props = $props();

    const validateMfaRecoveryCodeMutation =
        useValidateMfaRecoveryCode();

    async function handleSubmit(event: SubmitEvent): Promise<void> {
        event.preventDefault();
        const formData = new FormData(event.target as HTMLFormElement);
        const recoveryCode = (formData.get("recovery-code") as string)
            ?.trim();
        if (!recoveryCode) return;

        validateMfaRecoveryCodeMutation.mutate(recoveryCode);
    }
</script>

<LoginFormTitle title="2FA Recovery" {onBack} />
<p class="subtitle">
    No 2FA device available? <br>Paste your backup code below.
</p>

<form onsubmit={handleSubmit} class="recovery-form">
    <div class="form-group">
        <input
            type="text"
            name="recovery-code"
            id="recovery-code"
            class="form-input"
            placeholder="Enter your recovery code"
            disabled={validateMfaRecoveryCodeMutation.isPending}
            required
        />
    </div>
    {#if validateMfaRecoveryCodeMutation.isError}
        <div class="error-notification">
            <InlineNotification
                type="error"
                message={validateMfaRecoveryCodeMutation.error?.message
                || "Failed to validate recovery code"}
            />
        </div>
    {/if}
    <button
        type="submit"
        class="btn-primary"
        disabled={validateMfaRecoveryCodeMutation.isPending}
    >
        {
            validateMfaRecoveryCodeMutation.isPending
            ? "Verifying..."
            : "Continue"
        }
    </button>
</form>

<style>
    .subtitle {
      font-size: 1rem;
      color: #272626;
      line-height: 1.5;
      margin-bottom: 1rem;
    }

    .recovery-form {
      margin-bottom: 0.5rem;
    }

    .form-group {
      margin-bottom: 1.25rem;
      text-align: left;
    }

    .form-input {
      width: 100%;
      padding: 0.75rem 1rem;
      border: 1px solid #d1d5db;
      border-radius: 0.5rem;
      font-size: 1rem;
      background: white;
      box-sizing: border-box;
      transition: border-color 0.2s, box-shadow 0.2s;
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

    .error-notification {
      margin-bottom: 1.25rem;
    }
</style>
