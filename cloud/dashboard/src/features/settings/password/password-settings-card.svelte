<script lang="ts">
    import Modal from "$lib/components/modal.svelte";
    import SettingsCard from "$lib/components/settings-card.svelte";
    import ChangePasswordModal from "./change-password-modal.svelte";
    import SetPasswordModal from "./set-password-modal.svelte";

    interface Props {
        isLoading: boolean;
        hasPassword: boolean;
    }

    let { isLoading, hasPassword }: Props = $props();

    let changePasswordModal = $state<Modal | undefined>();
    let setPasswordModal = $state<Modal | undefined>();
</script>

<SettingsCard
    title="Password"
    badge={{
        label: hasPassword ? "Enabled" : "Disabled",
        variant: hasPassword ? "enabled" : "disabled",
    }}
    description="Make sure password is strong and secure. We recommend at least 15 characters and/or at least 8 characters including a number and a lowercase letters."
    {isLoading}
>
    <div class="password-actions">
        {#if hasPassword}
            <ChangePasswordModal bind:modal={changePasswordModal} />
            <button
                class="change-password-button"
                onclick={() => changePasswordModal?.openModal()}
            >
                Change password
            </button>
        {:else}
            <SetPasswordModal bind:modal={setPasswordModal} />
            <button
                class="set-password-button"
                onclick={() => setPasswordModal?.openModal()}
            >
                Set password
            </button>
        {/if}
    </div>
</SettingsCard>

<style>
    .password-actions {
      display: flex;
      justify-content: flex-start;
      width: fit-content;
      margin-top: 1rem;
    }

    .change-password-button,
    .set-password-button {
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

    .change-password-button:hover,
    .set-password-button:hover {
      background: var(--colors-gray50);
    }
</style>
