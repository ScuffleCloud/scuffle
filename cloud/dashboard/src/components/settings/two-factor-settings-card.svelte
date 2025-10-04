<script lang="ts">
    import SettingsCard from "$components/settings-card.svelte";
    import IconCheckSmall from "$lib/images/icon-check-small.svelte";
    import IconDots from "$lib/images/icon-dots.svelte";
    import IconLoginKey from "$lib/images/icon-login-key.svelte";
    import IconOverviewKey from "$lib/images/icon-overview-key.svelte";
    import { useWebauthnAuth } from "$lib/two-factor/webAuthn.svelte";
    import Modal from "../modal.svelte";
    export interface MfaMethod {
        id: string;
        name: string;
        type: "TOTP" | "WEBAUTH";
        isPrimary?: boolean;
    }
    // When something is updated we can just refetch the webauthn list. Methods will come from a tanstack query I guess
    interface Props {
        methods: MfaMethod[];
    }

    let { methods }: Props = $props();

    const enabled = $derived(methods.length > 0);

    // Flow states. Add one for TOPT later
    let currentStep = $state<"select" | "waiting" | "success">(
        "select",
    );
    let passkeyName = $state("");

    const stepToTitle = {
        select: "New 2FA method",
        waiting: "Waiting for results...",
        success: "Passkey added",
    };

    let modal: Modal;

    function handleReset() {
        passkeyName = "";
        currentStep = "select";
    }

    function handleInternalClose() {
        handleReset();
        modal.closeDialog();
    }

    const webauthnAuth = useWebauthnAuth();

    async function handlePasskeySetup() {
        currentStep = "waiting";
        try {
            await webauthnAuth.createCredential(
                passkeyName || "My Passkey",
            );
            // currentStep = "success";
        } catch (error) {
            console.log("an error has occured. Click to retry", error);
        }
    }

    function onEditMethod(id: string) {
        console.log("edit method", id);
    }
</script>

<SettingsCard
    title="Two-factor authentication"
    status={{
        label: enabled ? "Enabled" : "Disabled",
        variant: enabled ? "enabled" : "disabled",
    }}
    description="Enables a second layer of security, by requiring at least two methods of authentication for signing in."
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
                            {#if method.isPrimary}
                                <span class="primary-badge">Primary</span>
                            {/if}
                            <span class="method-type">{method.type}</span>
                        </div>
                    </div>
                    <button
                        class="icon-button"
                        onclick={() => onEditMethod(method.id)}
                        aria-label="Edit method"
                    >
                        <IconDots />
                    </button>
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
            title={stepToTitle[currentStep]}
            onClose={handleReset}
            hideCloseButton={currentStep === "waiting"}
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
                            <svg
                                width="20"
                                height="20"
                                viewBox="0 0 20 20"
                                fill="none"
                            >
                                <path
                                    d="M10 2L3 7V9C3 13.55 6.84 17.74 10 18C13.16 17.74 17 13.55 17 9V7L10 2Z"
                                    fill="currentColor"
                                />
                            </svg>
                            Continue with 2FA Code
                        </button>
                    </div>
                </div>
            {:else if currentStep === "waiting"}
                <div class="step-content centered">
                    <div class="spinner-container">
                        <div class="spinner"></div>
                    </div>

                    <button class="cancel-button" onclick={handleInternalClose}>
                        Cancel
                    </button>
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
                        placeholder="passkeyname"
                        class="passkey-input"
                    />

                    <button class="done-button" onclick={handleInternalClose}>
                        Done
                    </button>
                </div>
            {/if}
        </Modal>
    </div>
</SettingsCard>

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
    }

    .icon-button {
      background: none;
      border: none;
      padding: 0.25rem;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--colors-brown70);
      transition: color 0.2s;
    }

    .icon-button:hover {
      color: var(--colors-brown90);
    }

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

    .dialog-title {
      font-size: 1.875rem;
      font-weight: 700;
      color: rgb(23, 23, 23);
      margin: 0;
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

    .cancel-button {
      padding: 0.75rem 2rem;
      border-radius: 0.5rem;
      background-color: rgb(228, 228, 231);
      color: rgb(82, 82, 82);
      font-weight: 500;
      border: none;
      cursor: pointer;
    }

    .cancel-button:hover {
      background-color: rgb(212, 212, 216);
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
</style>
