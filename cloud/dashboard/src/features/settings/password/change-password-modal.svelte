<script lang="ts">
    import { authState } from "$lib/auth.svelte";
    import InlineNotification from "$lib/components/inline-notification.svelte";
    import Modal from "$lib/components/modal.svelte";
    import PasswordInput from "$lib/components/password-input.svelte";
    import { useUpdatePassword } from "./passwordMutations";

    interface Props {
        modal: Modal | undefined;
    }

    let { modal = $bindable() }: Props = $props();
    const userId = authState().user?.id;

    const updatePasswordMutation = useUpdatePassword(userId);

    let currentPassword = $state("");
    let newPassword = $state("");
    let confirmPassword = $state("");

    function handleClose() {
        currentPassword = "";
        newPassword = "";
        confirmPassword = "";
    }

    function handleSubmit() {
        updatePasswordMutation.mutate(
            { currentPassword, newPassword, confirmPassword },
            {
                onSuccess: () => {
                    modal?.closeModal();
                },
            },
        );
    }
</script>

<Modal
    title="Change password"
    onClose={handleClose}
    bind:this={modal}
    closeOnOutsideClick={!updatePasswordMutation.isPending}
>
    <div class="password-modal-content">
        <PasswordInput
            id="current-password"
            label="Current password"
            bind:value={currentPassword}
            placeholder="Enter current password"
            disabled={updatePasswordMutation.isPending}
        />
        <PasswordInput
            id="new-password"
            label="New password"
            bind:value={newPassword}
            placeholder="Enter new password"
            disabled={updatePasswordMutation.isPending}
        />
        <PasswordInput
            id="confirm-password"
            label="Confirm new password"
            bind:value={confirmPassword}
            placeholder="Confirm new password"
            disabled={updatePasswordMutation.isPending}
        />

        {#if updatePasswordMutation.error}
            <InlineNotification
                type="error"
                message={updatePasswordMutation.error.message}
            />
        {/if}

        <div class="button-group">
            <button
                class="done-button"
                onclick={handleSubmit}
                disabled={updatePasswordMutation.isPending
                || !currentPassword
                || !newPassword
                || !confirmPassword}
            >
                {
                    updatePasswordMutation.isPending
                    ? "Updating..."
                    : "Update password"
                }
            </button>
            <button
                class="default-button"
                onclick={() => modal?.closeModal()}
                disabled={updatePasswordMutation.isPending}
            >
                Cancel
            </button>
        </div>
    </div>
</Modal>

<style>
    .password-modal-content {
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }

    .button-group {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      margin-top: 0.5rem;
    }

    .done-button {
      padding: 0.875rem 1.5rem;
      border-radius: 0.75rem;
      background-color: rgb(252, 224, 172);
      color: rgb(65, 28, 9);
      font-weight: 600;
      font-size: 1rem;
      border: none;
      cursor: pointer;
      width: 100%;
    }

    .done-button:hover:not(:disabled) {
      background-color: rgb(249, 201, 120);
    }

    .done-button:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }

    .default-button {
      padding: 0.75rem 2rem;
      border-radius: 0.5rem;
      background-color: rgb(228, 228, 231);
      color: rgb(82, 82, 82);
      font-weight: 500;
      border: none;
      cursor: pointer;
      width: 100%;
    }

    .default-button:hover:not(:disabled) {
      background-color: rgb(212, 212, 216);
    }

    .default-button:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }
</style>
