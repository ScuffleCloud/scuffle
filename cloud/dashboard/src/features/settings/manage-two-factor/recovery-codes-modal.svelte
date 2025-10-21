<script lang="ts">
    import { authState } from "$lib/auth.svelte";
    import InlineNotification from "$lib/components/inline-notification.svelte";
    import Modal from "$lib/components/modal.svelte";
    import { usersServiceClient } from "$lib/grpcClient";
    import IconCopy from "$lib/images/icon-copy.svelte";
    import { createMutation } from "@tanstack/svelte-query";

    interface Props {
        modal: Modal | null;
        hasExistingCodes: boolean;
    }

    let { modal = $bindable(), hasExistingCodes }: Props = $props();

    const user = authState().user;
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
        modal?.closeModal();
    }

    function handleCopy() {
        navigator.clipboard.writeText(generatedCodes.join("\n"));
    }
</script>

<Modal
    bind:this={modal}
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

            <InlineNotification
                type="warning"
                message="Save these codes in a secure location. Each code can only be used once."
            />

            <div class="button-group">
                <button
                    class="button button-secondary"
                    onclick={() => handleDone()}
                    disabled={generateCodesMutation.isPending}
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
                    <IconCopy />
                    Copy All
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
