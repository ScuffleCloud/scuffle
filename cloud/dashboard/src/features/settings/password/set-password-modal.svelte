<script lang="ts">
    import { authState } from "$lib/auth.svelte";
    import InlineNotification from "$lib/components/inline-notification.svelte";
    import Modal from "$lib/components/modal.svelte";
    import PasswordInput from "$lib/components/password-input.svelte";
    import { useSetPassword } from "./passwordMutations";

    interface Props {
        modal: Modal | undefined;
    }

    let { modal = $bindable() }: Props = $props();

    const userId = authState().user?.id;
    const setPasswordMutation = useSetPassword(userId);

    let password = $state("");
    let confirmPassword = $state("");

    function handleClose() {
        password = "";
        confirmPassword = "";
    }

    function handleSubmit() {
        setPasswordMutation.mutate(
            { password, confirmPassword },
            {
                onSuccess: () => {
                    modal?.closeModal();
                },
            },
        );
    }
</script>

<Modal
    title="Set password"
    onClose={handleClose}
    bind:this={modal}
    closeOnOutsideClick={!setPasswordMutation.isPending}
>
    <div class="password-modal-content">
        <p class="description">
            Set a password for your account. Make sure password is strong and
            secure.
        </p>
        <PasswordInput
            id="password"
            label="Password"
            bind:value={password}
            placeholder="Enter password"
            disabled={setPasswordMutation.isPending}
        />
        <PasswordInput
            id="confirm-password"
            label="Confirm password"
            bind:value={confirmPassword}
            placeholder="Confirm password"
            disabled={setPasswordMutation.isPending}
        />
        {#if setPasswordMutation.error}
            <InlineNotification
                type="error"
                message={setPasswordMutation.error.message}
            />
        {/if}

        <button
            class="continue-button"
            onclick={handleSubmit}
            disabled={setPasswordMutation.isPending || !password
            || !confirmPassword}
        >
            {
                setPasswordMutation.isPending
                ? "Setting..."
                : "Continue"
            }
        </button>
    </div>
</Modal>

<style>
    .password-modal-content {
      display: flex;
      flex-direction: column;
      gap: 1.5rem;
    }

    .description {
      font-size: 1rem;
      color: rgb(82, 82, 82);
      margin: 0;
      line-height: 1.5;
      text-align: center;
    }

    .continue-button {
      padding: 1rem 1.5rem;
      border-radius: 0.75rem;
      background-color: rgb(252, 224, 172);
      color: rgb(65, 28, 9);
      font-weight: 600;
      font-size: 1.125rem;
      border: none;
      cursor: pointer;
      width: 100%;
      margin-top: 0.5rem;
    }

    .continue-button:hover:not(:disabled) {
      background-color: rgb(249, 201, 120);
    }

    .continue-button:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }
</style>
