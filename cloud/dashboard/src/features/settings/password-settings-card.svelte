<script lang="ts">
    import InlineNotification from "$lib/components/inline-notification.svelte";
    import Modal from "$lib/components/modal.svelte";
    import SettingsCard from "$lib/components/settings-card.svelte";

    interface Props {
        isLoading?: boolean;
    }

    let { isLoading }: Props = $props();

    // Password change state
    let currentPassword = $state("");
    let newPassword = $state("");
    let confirmPassword = $state("");
    let passwordError = $state("");
    let isUpdating = $state(false);

    let modal: Modal;

    function validatePassword(password: string): string | null {
        if (password.length < 8) {
            return "Password must be at least 8 characters";
        }
        if (password.length < 15 && !/\d/.test(password)) {
            return "Password must contain at least one number";
        }
        if (password.length < 15 && !/[a-z]/.test(password)) {
            return "Password must contain at least one lowercase letter";
        }
        return null;
    }

    function handlePasswordModalClose() {
        currentPassword = "";
        newPassword = "";
        confirmPassword = "";
        passwordError = "";
    }

    async function handlePasswordChange() {
        passwordError = "";

        if (!currentPassword || !newPassword || !confirmPassword) {
            passwordError = "All fields are required";
            return;
        }

        if (newPassword !== confirmPassword) {
            passwordError = "New passwords do not match";
            return;
        }

        const validationError = validatePassword(newPassword);
        if (validationError) {
            passwordError = validationError;
            return;
        }

        isUpdating = true;

        try {
            // TODO: Implement actual password change mutation
            // await updatePasswordMutation.mutate({
            //     currentPassword,
            //     newPassword
            // });

            // Simulate API call
            await new Promise(resolve => setTimeout(resolve, 1000));

            modal.closeModal();
        } catch (error) {
            passwordError = error instanceof Error
                ? error.message
                : "Failed to update password";
        } finally {
            isUpdating = false;
        }
    }
</script>

<SettingsCard
    title="Password"
    badge={{
        label: "Enabled",
        variant: "enabled",
    }}
    description="Make sure password is strong and secure. We recommend at least 15 characters and/or at least 8 characters including a number and a lowercase letters."
    {isLoading}
>
    <div class="password-actions">
        <Modal
            title="Change password"
            onClose={handlePasswordModalClose}
            bind:this={modal}
            closeOnOutsideClick={!isUpdating}
        >
            <div class="password-modal-content">
                <div class="input-group">
                    <label for="current-password">Current password</label>
                    <input
                        id="current-password"
                        type="password"
                        bind:value={currentPassword}
                        placeholder="Enter current password"
                        class="password-input"
                        disabled={isUpdating}
                    />
                </div>

                <div class="input-group">
                    <label for="new-password">New password</label>
                    <input
                        id="new-password"
                        type="password"
                        bind:value={newPassword}
                        placeholder="Enter new password"
                        class="password-input"
                        disabled={isUpdating}
                    />
                </div>

                <div class="input-group">
                    <label for="confirm-password">Confirm new password</label>
                    <input
                        id="confirm-password"
                        type="password"
                        bind:value={confirmPassword}
                        placeholder="Confirm new password"
                        class="password-input"
                        disabled={isUpdating}
                    />
                </div>

                {#if passwordError}
                    <InlineNotification
                        type="error"
                        message={passwordError}
                    />
                {/if}

                <div class="button-group">
                    <button
                        class="done-button"
                        onclick={handlePasswordChange}
                        disabled={isUpdating || !currentPassword
                        || !newPassword
                        || !confirmPassword}
                    >
                        {
                            isUpdating
                            ? "Updating..."
                            : "Update password"
                        }
                    </button>
                    <button
                        class="default-button"
                        onclick={() => modal.closeModal()}
                        disabled={isUpdating}
                    >
                        Cancel
                    </button>
                </div>
            </div>
        </Modal>
        <div class="change-password-container">
            <button
                class="change-password-button"
                onclick={() => modal.openModal()}
            >
                Change password
            </button>
        </div>
    </div>
</SettingsCard>

<style>
    .password-actions {
      display: flex;
    }

    .password-actions :global(.change-password-button) {
      width: 100%;
      padding: 0.75rem;
      background: var(--colors-gray40);
      border: none;
      border-radius: 0.5rem;
      font-size: 1rem;
      font-weight: 600;
      color: var(--colors-brown90);
      cursor: pointer;
      transition: background 0.2s;
    }

    .password-actions :global(.change-password-button:hover) {
      background: var(--colors-gray50);
    }

    .password-modal-content {
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }

    .input-group {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }

    .input-group label {
      font-size: 0.875rem;
      font-weight: 600;
      color: var(--colors-brown90);
    }

    .password-input {
      padding: 0.875rem 1rem;
      border: 1px solid rgb(212, 212, 216);
      border-radius: 0.5rem;
      font-size: 1rem;
      background-color: white;
    }

    .password-input:focus {
      outline: none;
      border-color: rgb(247, 177, 85);
    }

    .password-input:disabled {
      background-color: rgb(243, 244, 246);
      cursor: not-allowed;
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

    .change-password-container {
      display: flex;
    }

    .change-password-button {
      width: fit-content;
      padding: 0.75rem 2rem;
      border-radius: 0.5rem;
      background-color: rgb(228, 228, 231);
      color: rgb(82, 82, 82);
      font-weight: 500;
      border: none;
      cursor: pointer;
    }
</style>
