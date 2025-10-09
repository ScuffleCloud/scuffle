<!-- TODO: Refactor this component, add verification of user identity (password/2fa code) -->
<script lang="ts">
    import Modal from "$components/modal.svelte";
    import { authState } from "$lib/auth.svelte";
    import { usersServiceClient } from "$lib/grpcClient";
    import { createMutation } from "@tanstack/svelte-query";

    interface Props {
        enabled: boolean;
        hasExistingCodes: boolean;
    }

    let { hasExistingCodes }: Props = $props();

    const user = authState().user;
    let modal: Modal;
    let currentStep = $state<"confirm" | "success">("confirm");
    let generatedCodes = $state<string[]>([]);

    const generateCodesMutation = createMutation(() => ({
        mutationFn: async () =>
            await usersServiceClient.regenerateRecoveryCodes({
                id: user!.id,
            }),
        onSuccess: ({ response: { codes } }) => {
            generatedCodes = codes;
            currentStep = "success";
        },
    }));

    function handleConfirm() {
        generateCodesMutation.mutate();
    }

    function handleDone() {
        currentStep = "confirm";
        generatedCodes = [];
        modal.closeDialog();
    }

    function handleCopy() {
        navigator.clipboard.writeText(generatedCodes.join("\n"));
    }

    function handleDownload() {
        const blob = new Blob([generatedCodes.join("\n")], {
            type: "text/plain",
        });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = "recovery-codes.txt";
        a.click();
        URL.revokeObjectURL(url);
    }
</script>

<Modal
    bind:this={modal}
    triggerLabel={hasExistingCodes ? "Regenerate Codes" : "Generate Codes"}
    triggerClass="action-button action-secondary"
    title={currentStep === "confirm"
    ? "Generate Recovery Codes"
    : "Your Recovery Codes"}
    onClose={() => {
        currentStep = "confirm";
        generatedCodes = [];
    }}
>
    {#if currentStep === "confirm"}
        <div class="modal-content">
            <p class="warning-text">
                {#if hasExistingCodes}
                    Generating new recovery codes will invalidate your existing
                    codes. Make sure you've saved them somewhere safe.
                {:else}
                    Recovery codes are used to access your account if you lose
                    access to your two-factor authentication device.
                {/if}
            </p>

            <div class="warning-box">
                <p class="warning-title">Important</p>
                <p class="warning-description">
                    Save these codes in a secure location. Each code can only be
                    used once.
                </p>
            </div>

            <div class="button-group">
                <button
                    class="button button-secondary"
                    onclick={() => modal.closeDialog()}
                >
                    Cancel
                </button>
                <button
                    class="button button-primary"
                    onclick={handleConfirm}
                    disabled={generateCodesMutation.isPending}
                >
                    {
                        generateCodesMutation.isPending
                        ? "Generating..."
                        : "Generate Codes"
                    }
                </button>
            </div>

            {#if generateCodesMutation.isError}
                <p class="error-text">
                    Failed to generate recovery codes. Please try again.
                </p>
            {/if}
        </div>
    {:else if currentStep === "success"}
        <div class="modal-content">
            <p class="success-text">
                Your recovery codes have been generated. Store them in a safe
                place.
            </p>

            <div class="codes-container">
                {#each generatedCodes as code, index (`code-${index + 1}`)}
                    <code class="recovery-code">{code}</code>
                {/each}
            </div>

            <div class="action-buttons">
                <button class="button button-secondary" onclick={handleCopy}>
                    <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                        <path
                            d="M5.5 4.5H3.5C2.94772 4.5 2.5 4.94772 2.5 5.5V12.5C2.5 13.0523 2.94772 13.5 3.5 13.5H10.5C11.0523 13.5 11.5 13.0523 11.5 12.5V10.5M5.5 10.5H12.5C13.0523 10.5 13.5 10.0523 13.5 9.5V2.5C13.5 1.94772 13.0523 1.5 12.5 1.5H5.5C4.94772 1.5 4.5 1.94772 4.5 2.5V9.5C4.5 10.0523 4.94772 10.5 5.5 10.5Z"
                            stroke="currentColor"
                            stroke-width="1.5"
                            stroke-linecap="round"
                        />
                    </svg>
                    Copy All
                </button>
                <button
                    class="button button-secondary"
                    onclick={handleDownload}
                >
                    <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                        <path
                            d="M8 10.5V2.5M8 10.5L10.5 8M8 10.5L5.5 8M13.5 10.5V12.5C13.5 13.0523 13.0523 13.5 12.5 13.5H3.5C2.94772 13.5 2.5 13.0523 2.5 12.5V10.5"
                            stroke="currentColor"
                            stroke-width="1.5"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        />
                    </svg>
                    Download
                </button>
            </div>

            <button
                class="button button-primary full-width"
                onclick={handleDone}
            >
                Done
            </button>
        </div>
    {/if}
</Modal>

<style>
    .modal-content {
      display: flex;
      flex-direction: column;
      gap: 1.5rem;
    }

    .warning-text {
      font-size: 1rem;
      color: rgb(82, 82, 82);
      line-height: 1.5;
      margin: 0;
    }

    .warning-box {
      display: flex;
      gap: 0.75rem;
      padding: 1rem;
      background-color: rgb(254, 243, 199);
      border-radius: 0.5rem;
      border: 1px solid rgb(251, 191, 36);
    }

    .warning-title {
      font-weight: 600;
      color: rgb(146, 64, 14);
      margin: 0 0 0.25rem 0;
    }

    .warning-description {
      font-size: 0.875rem;
      color: rgb(146, 64, 14);
      margin: 0;
    }

    .success-text {
      font-size: 1rem;
      color: rgb(20, 83, 45);
      background-color: rgb(187, 247, 208);
      padding: 0.75rem 1rem;
      border-radius: 0.5rem;
      margin: 0;
    }

    .codes-container {
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 0.5rem;
      padding: 1rem;
      background-color: rgb(250, 250, 250);
      border-radius: 0.5rem;
      border: 1px solid rgb(228, 228, 231);
    }

    .recovery-code {
      font-family: monospace;
      font-size: 0.875rem;
      padding: 0.5rem;
      background-color: white;
      border: 1px solid rgb(228, 228, 231);
      border-radius: 0.25rem;
      text-align: center;
    }

    .button-group {
      display: flex;
      gap: 0.75rem;
      justify-content: flex-end;
    }

    .action-buttons {
      display: flex;
      gap: 0.5rem;
    }

    .button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
      padding: 0.75rem 1.5rem;
      border-radius: 0.5rem;
      font-weight: 600;
      font-size: 1rem;
      border: none;
      cursor: pointer;
      transition: all 0.2s;
    }

    .button:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }

    .button-primary {
      background-color: rgb(252, 224, 172);
      color: rgb(65, 28, 9);
    }

    .button-primary:hover:not(:disabled) {
      background-color: rgb(249, 201, 120);
    }

    .button-secondary {
      background-color: rgb(228, 228, 231);
      color: rgb(82, 82, 82);
    }

    .button-secondary:hover:not(:disabled) {
      background-color: rgb(212, 212, 216);
    }

    .full-width {
      width: 100%;
    }

    .error-text {
      font-size: 0.875rem;
      color: rgb(185, 28, 28);
      margin: 0;
      text-align: center;
    }

    :global(.action-button) {
      padding: 0.75rem 1.5rem;
      border-radius: 0.5rem;
      font-weight: 600;
      border: none;
      cursor: pointer;
      transition: background 0.2s;
    }

    :global(.action-button.action-secondary) {
      background: var(--colors-gray40, #e5e5e5);
      color: var(--colors-brown90, #1c1917);
    }

    :global(.action-button.action-secondary:hover:not(:disabled)) {
      background: var(--colors-gray50, #d4d4d4);
    }

    :global(.action-button:disabled) {
      opacity: 0.5;
      cursor: not-allowed;
    }
</style>
