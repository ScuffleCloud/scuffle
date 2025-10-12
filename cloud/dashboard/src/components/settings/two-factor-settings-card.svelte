<script lang="ts">
    import InlineNotification from "$components/inline-notification.svelte";
    import SettingsCard from "$components/settings-card.svelte";
    import { authState } from "$lib/auth.svelte";
    import IconCheckSmall from "$lib/images/icon-check-small.svelte";
    import IconLoginKey from "$lib/images/icon-login-key.svelte";
    import IconOverviewKey from "$lib/images/icon-overview-key.svelte";
    import IconShield from "$lib/images/icon-shield.svelte";
    import type { MfaCredential } from "$lib/types";
    import { useQueryClient } from "@tanstack/svelte-query";

    import Badge from "$components/badge.svelte";
    import Modal from "../modal.svelte";
    import AuthMethodActionsMenu from "./auth-method-actions-menu.svelte";
    import {
        type AuthStepType,
        DEFAULT_WEBAUTHN_AUTH_NAME,
        STEP_TO_TITLE,
    } from "./consts";
    import {
        useCreateWebauthnCredential,
        useDeleteWebauthnCredential,
        useUpdateWebauthnName,
    } from "./credentialMutations.svelte";
    interface Props {
        methods: MfaCredential[];
        isLoading?: boolean;
    }

    let { methods, isLoading }: Props = $props();

    const queryClient = useQueryClient();
    const userId = authState().user?.id;
    const enabled = $derived(methods.length > 0);

    // Flow states. Add one for TOPT later
    let currentStep = $state<AuthStepType>(
        "select",
    );

    // Webauthn setup flow
    let credentialId = $state("");
    let passkeyName = $state("");

    // Edit state
    let editingCredentialId = $state<string | null>(null);
    let editingName = $state("");

    // Delete state
    let methodToDelete = $state<MfaCredential | null>(null);

    let modal: Modal;
    let editModal: Modal;
    let deleteModal: Modal;

    // --Mutations--
    const createWebAuthnMutation = useCreateWebauthnCredential(userId);
    const updateNameMutation = useUpdateWebauthnName(userId);
    const deleteCredentialMutation = useDeleteWebauthnCredential(
        userId,
    );

    // --Webauthn setup flow--
    function handleReset() {
        // If exited in the last step without naming then refetch the list
        if (currentStep === "success") {
            queryClient.invalidateQueries({
                queryKey: ["webauthn-list"],
            });
        }
        passkeyName = "";
        credentialId = "";
        currentStep = "select";
        createWebAuthnMutation.reset();
        updateNameMutation.reset();
    }

    function handlePasskeySetup() {
        currentStep = "waiting";
        createWebAuthnMutation.mutate(undefined, {
            onSuccess: (id) => {
                credentialId = id;
                currentStep = "success";
            },
        });
    }

    function handleRetry() {
        // Clear errror before retrying
        createWebAuthnMutation.reset();
        createWebAuthnMutation.mutate();
    }

    function handlePasskeyNameSubmit() {
        if (passkeyName.trim()) {
            updateNameMutation.mutate({
                id: credentialId,
                name: passkeyName.trim(),
            }, {
                onSuccess: () => {
                    modal.closeModal();
                },
            });
        }
    }
    // --End Webauthn setup flow--

    // --Webauthn edit modals--
    function onEditMethod(method: MfaCredential) {
        editingCredentialId = method.id;
        editingName = method.name || "";
        editModal.openModal();
    }

    function handleEditModalClose() {
        editingCredentialId = null;
        editingName = "";
        updateNameMutation.reset();
    }

    function handleEditModalSubmit() {
        if (editingCredentialId && editingName.trim()) {
            updateNameMutation.mutate({
                id: editingCredentialId,
                name: editingName.trim(),
            }, {
                onSuccess: () => {
                    editModal.closeModal();
                },
            });
        }
    }
    // --End Webauthn edit modals--

    // --Webauthn delete modals--
    function onDeleteMethod(method: MfaCredential) {
        methodToDelete = method;
        deleteModal.openModal();
    }

    function handleDeleteModalClose() {
        methodToDelete = null;
        deleteCredentialMutation.reset();
    }

    function handleDeleteConfirm() {
        if (methodToDelete) {
            deleteCredentialMutation.mutate({ id: methodToDelete.id }, {
                onSuccess: () => {
                    deleteModal.closeModal();
                },
            });
        }
    }
    // --End Webauthn delete modals--
</script>
<SettingsCard
    title="Two-factor authentication"
    badge={{
        label: enabled ? "Enabled" : "Disabled",
        variant: enabled ? "enabled" : "disabled",
    }}
    description="Enables a second layer of security, by requiring at least two methods of authentication for signing in."
    {isLoading}
>
    <div class="divider">
        Active authentication methods
        <div class="divider-line"></div>
    </div>
    {#if enabled && methods.length > 0}
        <div class="methods-list">
            {#each methods as method (method.id)}
                <div class="method-item">
                    <div class="method-icon">
                        <IconOverviewKey />
                    </div>
                    <div class="method-info">
                        <div class="method-name-row">
                            <span class="method-name">{method.name}</span>
                            <Badge variant="info">{
                                method.type.toUpperCase()
                            }</Badge>
                        </div>
                    </div>
                    <AuthMethodActionsMenu
                        onEdit={() => onEditMethod(method)}
                        onDelete={() => onDeleteMethod(method)}
                    />
                </div>
            {/each}
        </div>
    {:else}
        <div class="no-active-methods">
            No active 2FA methods.
        </div>
    {/if}

    <div class="add-method-button-container">
        <Modal
            triggerLabel="Add a method"
            triggerClass="add-method-button"
            title={STEP_TO_TITLE[currentStep]}
            onClose={handleReset}
            hideCloseButton={currentStep === "waiting"}
            closeOnOutsideClick={currentStep === "select"}
            bind:this={modal}
        >
            {#if currentStep === "select"}
                <div class="step-content">
                    <p class="dialog-description">
                        Choose new method for authentication
                    </p>

                    <div class="methods">
                        <button
                            class="method-button primary"
                            onclick={handlePasskeySetup}
                        >
                            <IconLoginKey />
                            Continue with Passkey
                        </button>

                        <button class="method-button secondary" disabled>
                            <IconShield />
                            Continue with 2FA Code
                        </button>
                    </div>
                </div>
            {:else if currentStep === "waiting"}
                <div class="step-content centered">
                    {#if createWebAuthnMutation.isError}
                        <InlineNotification
                            type="error"
                            message={createWebAuthnMutation.error?.message
                            || "An error occurred"}
                        />
                        <div class="button-group">
                            <button
                                class="default-button"
                                onclick={handleRetry}
                            >
                                Send request again
                            </button>
                            <button
                                class="default-button"
                                onclick={() => modal.closeModal()}
                            >
                                Cancel
                            </button>
                        </div>
                    {:else}
                        <div class="spinner-container">
                            <div class="spinner"></div>
                        </div>
                        <!-- TBD on a cancel button here too, it won't stop the navigator.credentials.create from firing even if it hangs. -->
                    {/if}
                </div>
            {:else if currentStep === "success"}
                <div class="step-content">
                    <div class="success-message">
                        <IconCheckSmall />
                        New passkey successfully added
                    </div>

                    <p class="optional-text">
                        Optionally you can name the passkey.
                    </p>

                    <input
                        type="text"
                        bind:value={passkeyName}
                        placeholder={DEFAULT_WEBAUTHN_AUTH_NAME}
                        class="passkey-input"
                    />
                    {#if updateNameMutation.isError}
                        <InlineNotification
                            type="error"
                            message={updateNameMutation.error?.message
                            || "Failed to update name"}
                        />
                    {/if}
                    <button
                        class="done-button"
                        onclick={handlePasskeyNameSubmit}
                    >
                        Done
                    </button>
                </div>
            {/if}
        </Modal>
    </div>
</SettingsCard>

<Modal
    title="Edit passkey name"
    onClose={handleEditModalClose}
    bind:this={editModal}
    closeOnOutsideClick={!updateNameMutation.isPending}
>
    <div class="edit-modal-content">
        <input
            type="text"
            bind:value={editingName}
            placeholder="Passkey name"
            class="passkey-input"
            disabled={updateNameMutation.isPending}
        />

        {#if updateNameMutation.isError}
            <InlineNotification
                type="error"
                message={updateNameMutation.error?.message
                || "Failed to update name"}
            />
        {/if}

        <div class="button-group">
            <button
                class="done-button"
                onclick={handleEditModalSubmit}
                disabled={updateNameMutation.isPending
                || !editingName.trim()}
            >
                {
                    updateNameMutation.isPending
                    ? "Saving..."
                    : "Save"
                }
            </button>
            <button
                class="default-button"
                onclick={() => editModal.closeModal()}
                disabled={updateNameMutation.isPending}
            >
                Cancel
            </button>
        </div>
    </div>
</Modal>

<!-- Delete confirmation modal -->
<Modal
    title="Delete passkey?"
    onClose={handleDeleteModalClose}
    bind:this={deleteModal}
    closeOnOutsideClick={!deleteCredentialMutation.isPending}
>
    <div class="delete-modal-content">
        <p class="delete-warning">
            Are you sure you want to delete <strong>{
                methodToDelete?.name ?? "this passkey"
            }</strong>? This action cannot be undone.
        </p>

        {#if deleteCredentialMutation.isError}
            <InlineNotification
                type="error"
                message={deleteCredentialMutation.error?.message
                || "Failed to delete"}
            />
        {/if}

        <div class="button-group">
            <button
                class="danger-button"
                onclick={handleDeleteConfirm}
                disabled={deleteCredentialMutation.isPending}
            >
                {
                    deleteCredentialMutation.isPending
                    ? "Deleting..."
                    : "Delete"
                }
            </button>
            <button
                class="default-button"
                onclick={() => deleteModal.closeModal()}
                disabled={deleteCredentialMutation.isPending}
            >
                Cancel
            </button>
        </div>
    </div>
</Modal>

<style>
    .divider {
      display: flex;
      align-items: center;
      gap: 0.375rem;
      color: var(--text-3, #645c59);
      font-size: 0.875rem;
      font-style: normal;
      font-weight: 600;
      line-height: 1rem;
      padding: 0.75rem 0;

      .divider-line {
        flex: 1;
        height: 1px;
        background: var(--alpha-dark-10, rgba(24, 23, 22, 0.05));
      }
    }

    .methods-list {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      margin-bottom: 1rem;
    }

    .method-item {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      border-radius: 0.5rem;
    }

    .method-icon {
      display: flex;
      align-items: center;
      justify-content: center;
      background: var(--gray-40, #f1eae7);
      border-radius: 0.5rem;
      padding: 0.75rem;
    }

    .method-info {
      flex: 1;
    }

    .method-name-row {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      flex-wrap: wrap;
    }

    .method-name {
      font-size: 1rem;
      font-weight: 600;
      color: var(--colors-brown90);
    }

    /*
        .primary-badge {
          font-size: 0.75rem;
          font-weight: 700;
          padding: 0.125rem 0.5rem;
          border-radius: 5.25rem;
          background: #fef3c7;
          color: #d97706;
        }

        .method-type {
          font-size: 0.875rem;
          font-weight: 600;
          padding: 0.125rem 0.5rem;
          border-radius: 0.25rem;
          background: var(--colors-gray50);
          color: var(--colors-brown70);
        } */

    .add-method-button-container {
      display: flex;
      justify-content: center;
      align-items: center;
      margin-top: 1rem;
    }

    .add-method-button-container :global(.add-method-button) {
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

    .add-method-button-container :global(.add-method-button:hover) {
      background: var(--colors-gray50);
    }

    .no-active-methods {
      display: flex;
      padding: 0.75rem 1.5rem;
      justify-content: center;
      align-items: center;
      gap: 0.625rem;

      color: var(--text-3, #645c59);
      text-align: center;
      font-size: 1rem;
      font-style: normal;
      font-weight: 500;
      line-height: 1.5rem;
    }

    /* Modal Content Styles */
    .step-content {
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }

    .step-content.centered {
      align-items: center;
      text-align: center;
    }

    .dialog-description {
      font-size: 1rem;
      color: rgb(82, 82, 82);
      margin: 0;
    }

    .methods {
      display: flex;
      flex-direction: column;
      gap: 1rem;
      margin-top: 0.5rem;
    }

    .method-button {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.75rem;
      padding: 1rem 1.5rem;
      border-radius: 0.75rem;
      font-size: 1.125rem;
      font-weight: 600;
      border: none;
      cursor: pointer;
      transition: all 0.2s;
    }

    .method-button.primary {
      background-color: rgb(255, 255, 255);
      color: rgb(23, 23, 23);
      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    }

    .method-button.primary:hover {
      box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    }

    .method-button.secondary {
      background-color: rgb(228, 228, 231);
      color: rgb(161, 161, 170);
    }

    .method-button:disabled {
      cursor: not-allowed;
    }

    .spinner-container {
      padding: 2rem 0;
    }

    .spinner {
      width: 3rem;
      height: 3rem;
      border: 4px solid rgb(228, 228, 231);
      border-top-color: rgb(23, 23, 23);
      border-radius: 50%;
      animation: spin 1s linear infinite;
    }

    @keyframes spin {
      to {
        transform: rotate(360deg);
      }
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

    .default-button:hover {
      background-color: rgb(212, 212, 216);
    }

    .button-group {
      display: flex;
      gap: 0.5rem;
      display: flex;
      flex-direction: column;
      width: 100%;
    }

    .success-message {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.75rem;
      padding: 0.75rem 0;
      background-color: rgb(187, 247, 208);
      border-radius: 0.75rem;
      color: rgb(20, 83, 45);
      font-weight: 500;
    }

    .optional-text {
      font-size: 0.875rem;
      color: rgb(82, 82, 82);
      margin: 0;
    }

    .passkey-input {
      padding: 0.875rem 1rem;
      border: 1px solid rgb(212, 212, 216);
      border-radius: 0.5rem;
      font-size: 1rem;
      background-color: white;
    }

    .passkey-input:focus {
      outline: none;
      border-color: rgb(247, 177, 85);
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
    }

    .done-button:hover {
      background-color: rgb(249, 201, 120);
    }

    .edit-modal-content,
    .delete-modal-content {
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }

    .danger-button {
      padding: 0.875rem 1.5rem;
      border-radius: 0.75rem;
      background-color: #fee;
      color: #7f1d1d;
      font-weight: 600;
      font-size: 1rem;
      border: 2px solid #fcc;
      cursor: pointer;
      width: 100%;
    }

    .danger-button:hover:not(:disabled) {
      background-color: #fcc;
    }

    .danger-button:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }
</style>
