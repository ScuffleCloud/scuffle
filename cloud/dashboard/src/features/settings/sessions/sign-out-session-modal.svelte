<script lang="ts">
    import { authState } from "$lib/auth.svelte";
    import Modal from "$lib/components/modal.svelte";
    import type { UserSession } from "@scufflecloud/proto/scufflecloud/core/v1/sessions.js";

    import { useRemoveSession } from "./mutations.svelte";

    interface Props {
        modal: Modal;
        session: UserSession;
    }

    let { modal = $bindable(), session }: Props = $props();

    const user = authState().user;

    const signOutMutation = useRemoveSession(user?.id);

    function handleSignOut() {
        signOutMutation.mutate({
            id: user!.id,
            deviceFingerprint: session.deviceFingerprint,
        }, {
            onSuccess: () => {
                modal?.closeModal();
            },
        });
    }
</script>

<Modal bind:this={modal}>
    <div class="sign-out-modal">
        <h2>Sign out session</h2>

        {#if session}
            <!-- <p class="modal-description">
                Are you sure you want to sign out? the session from <strong>{
                    session.location
                }</strong>
                ({session.ipAddress})? You will need to sign in again on that
                device.
            </p> -->
            <p class="modal-description">Are you sure you want to sign out?</p>

            <!-- <div class="session-info">
                <div class="info-row">
                    <span class="label">Device:</span>
                    <span class="value">{session.deviceType}</span>
                </div>
                <div class="info-row">
                    <span class="label">Browser:</span>
                    <span class="value">{session.browser}</span>
                </div>
                <div class="info-row">
                    <span class="label">Last active:</span>
                    <span class="value">{session.lastActive}</span>
                </div>
            </div> -->
        {/if}

        <div class="modal-actions">
            <button
                class="cancel-button"
                onclick={() => modal?.closeModal()}
                disabled={signOutMutation.isPending}
            >
                Cancel
            </button>
            <button
                class="sign-out-button"
                onclick={handleSignOut}
                disabled={signOutMutation.isPending}
            >
                {
                    signOutMutation.isPending
                    ? "Signing out..."
                    : "Sign out"
                }
            </button>
        </div>
    </div>
</Modal>

<style>
    .sign-out-modal {
      padding: 1.5rem;
      max-width: 500px;
    }

    h2 {
      margin: 0 0 1rem 0;
      font-size: 1.5rem;
      color: var(--colors-brown90);
    }

    .modal-description {
      margin-bottom: 1.5rem;
      color: var(--colors-brown70);
      line-height: 1.5;
    }

    .session-info {
      background: var(--colors-gray30);
      padding: 1rem;
      border-radius: 0.5rem;
      margin-bottom: 1.5rem;
    }

    .info-row {
      display: flex;
      justify-content: space-between;
      padding: 0.5rem 0;
    }

    .info-row:not(:last-child) {
      border-bottom: 1px solid var(--colors-gray50);
    }

    .label {
      color: var(--colors-brown60);
      font-weight: 600;
    }

    .value {
      color: var(--colors-brown80);
    }

    .modal-actions {
      display: flex;
      gap: 0.75rem;
      justify-content: flex-end;
    }

    .cancel-button,
    .sign-out-button {
      padding: 0.75rem 1.5rem;
      border: none;
      border-radius: 0.5rem;
      font-size: 1rem;
      font-weight: 600;
      cursor: pointer;
      transition: all 0.2s;
    }

    .cancel-button {
      background: var(--colors-gray40);
      color: var(--colors-brown80);
    }

    .cancel-button:hover:not(:disabled) {
      background: var(--colors-gray50);
    }

    .sign-out-button {
      background: var(--colors-red60);
      color: white;
    }

    .sign-out-button:hover:not(:disabled) {
      background: var(--colors-red70);
    }

    button:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
</style>
