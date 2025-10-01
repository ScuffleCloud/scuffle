<script lang="ts">
    import { rpcErrorToString, sessionsServiceClient } from "$lib/grpcClient";
    import IconArrowLeft from "$lib/images/icon-arrow-left.svelte";
    import { type RpcError } from "@protobuf-ts/runtime-rpc";

    interface Props {
        onBack: () => void;
    }

    let { onBack }: Props = $props();

    let loading = $state(false);
    let error = $state<string | null>(null);

    async function handleSubmit(event: SubmitEvent): Promise<void> {
        event.preventDefault();
        const formData = new FormData(event.target as HTMLFormElement);
        const recoveryCode = (formData.get("recovery-code") as string)
            .trim();

        if (!recoveryCode) {
            return;
        }

        loading = true;
        error = null;

        try {
            const validateCall = sessionsServiceClient
                .validateMfaForUserSession({
                    response: {
                        oneofKind: "recoveryCode",
                        recoveryCode: {
                            code: recoveryCode.trim(),
                        },
                    },
                });
            await validateCall.status;
        } catch (err) {
            const errorText = rpcErrorToString(err as RpcError);
            // TODO: Remove later after UI on how we want to show errors
            console.log(errorText);
            error = "Failed to validate recovery code";
        } finally {
            loading = false;
        }
    }
</script>

<div class="header">
    <button type="button" onclick={onBack} class="back-button">
        <IconArrowLeft />
    </button>
    <h1 class="title">2FA Recovery</h1>
</div>
<p class="subtitle">
    No 2FA device available? <br>Paste your backup code below.
</p>

{#if error}
    <div class="error-message">{error}</div>
{/if}

<form onsubmit={handleSubmit} class="recovery-form">
    <div class="form-group">
        <input
            type="text"
            name="recovery-code"
            id="recovery-code"
            class="form-input"
            placeholder="Enter your recovery code"
            disabled={loading}
            required
        />
    </div>
    <button type="submit" class="btn-primary" disabled={loading}>
        {loading ? "Verifying..." : "Continue"}
    </button>
</form>

<style>
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
</style>
