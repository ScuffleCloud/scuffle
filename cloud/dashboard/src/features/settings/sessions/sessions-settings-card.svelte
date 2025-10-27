<script lang="ts">
    import { authState } from "$lib/auth.svelte";
    import Modal from "$lib/components/modal.svelte";
    import SettingsCard from "$lib/components/settings-card.svelte";
    import type { UserSession } from "@scufflecloud/proto/scufflecloud/core/v1/sessions.js";
    import SignOutSessionModal from "./sign-out-session-modal.svelte";

    interface Props {
        isLoading?: boolean;
        sessions: UserSession[];
    }

    let { isLoading = false, sessions }: Props = $props();

    let signOutModal = $state<Modal | undefined>();
    let selectedSession = $state<UserSession | undefined>();

    function handleSignOut(session: UserSession) {
        selectedSession = session;
        signOutModal?.openModal();
    }

    function formatLastActive(lastActive: string) {
        const date = new Date(lastActive);
        const now = new Date();
        const diffMs = now.getTime() - date.getTime();
        const diffMins = Math.floor(diffMs / 60000);
        const diffHours = Math.floor(diffMs / 3600000);
        const diffDays = Math.floor(diffMs / 86400000);

        if (diffMins < 60) {
            return `${diffMins} minutes ago`;
        } else if (diffHours < 24) {
            return `${diffHours} hours ago`;
        } else if (diffDays < 7) {
            return `${diffDays} days ago`;
        } else {
            return date.toLocaleDateString("en-US", {
                month: "short",
                day: "numeric",
                year: "numeric",
            });
        }
    }
</script>

<SettingsCard
    title="Sessions"
    description="If you come across any sessions that seem unfamiliar or untrustworthy, please take a moment to end them immediately to ensure your security."
    {isLoading}
>
    <div class="sessions-container">
        <div class="sessions-header">
            <h3>Active sessions</h3>
        </div>

        <div class="sessions-list">
            {#each sessions as session, index (index)}
                <div class="session-item">
                    <div class="session-icon">
                        oijojo
                    </div>

                    <div class="session-details">
                        <div class="session-location">
                            <strong>{session.lastUsedAt}</strong>
                        </div>
                        <div class="session-ip">{session.lastIp}</div>
                    </div>

                    <div class="session-device">
                        <div class="device-type">
                            {
                                session
                                .deviceFingerprint
                            }
                        </div>
                        <!-- <div class="device-browser">{session.browser}</div> -->
                    </div>

                    <!-- <div class="session-time">
                        {
                            formatLastActive(
                                session.lastUsedAt,
                            )
                        }
                    </div> -->

                    <div class="session-status">
                        <!-- {#if session.isCurrent} -->
                        <span class="current-badge">Current</span>
                        <!-- {:else} -->
                        <!-- <button
                                class="sign-out-button"
                                onclick={() => handleSignOut(session)}
                            >
                                Sign out
                            </button>
                        {/if} -->
                    </div>
                </div>
            {/each}
        </div>
    </div>

    <!-- <SignOutSessionModal
        bind:modal={signOutModal}
        session={selectedSession}
    /> -->
</SettingsCard>

<style>
    .sessions-container {
      margin-top: 1rem;
    }

    .sessions-header {
      margin-bottom: 1.5rem;
    }

    .sessions-header h3 {
      font-size: 1.25rem;
      font-weight: 700;
      margin: 0;
      color: var(--colors-brown90);
    }

    .sessions-list {
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }

    .session-item {
      display: grid;
      grid-template-columns: auto 1fr auto auto auto;
      align-items: center;
      gap: 1.5rem;
      padding: 1.25rem;
      background: var(--colors-gray30);
      border-radius: 0.75rem;
      transition: background 0.2s;
    }

    .session-item:hover {
      background: var(--colors-gray40);
    }

    .session-icon {
      font-size: 1.75rem;
      display: flex;
      align-items: center;
      justify-content: center;
      width: 3rem;
      height: 3rem;
      background: var(--colors-gray50);
      border-radius: 0.5rem;
    }

    .session-details {
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
      min-width: 200px;
    }

    .session-location {
      font-size: 1rem;
      color: var(--colors-brown90);
    }

    .session-ip {
      font-size: 0.875rem;
      color: var(--colors-brown60);
    }

    .session-device {
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
      min-width: 150px;
    }

    .device-type,
    .device-browser {
      font-size: 0.875rem;
      color: var(--colors-brown70);
    }

    .session-time {
      font-size: 0.875rem;
      color: var(--colors-brown60);
      min-width: 120px;
      text-align: right;
    }

    .session-status {
      min-width: 100px;
      display: flex;
      justify-content: flex-end;
    }

    .current-badge {
      padding: 0.5rem 1rem;
      background: var(--colors-gray50);
      border-radius: 0.375rem;
      font-size: 0.875rem;
      font-weight: 600;
      color: var(--colors-brown70);
    }

    .sign-out-button {
      padding: 0.5rem 1rem;
      background: transparent;
      border: 1px solid var(--colors-gray60);
      border-radius: 0.375rem;
      font-size: 0.875rem;
      font-weight: 600;
      color: var(--colors-brown80);
      cursor: pointer;
      transition: all 0.2s;
    }

    .sign-out-button:hover {
      background: var(--colors-gray50);
      border-color: var(--colors-gray70);
    }

    @media (max-width: 768px) {
      .session-item {
        grid-template-columns: auto 1fr;
        gap: 1rem;
      }

      .session-device,
      .session-time {
        grid-column: 2;
      }

      .session-status {
        grid-column: 1 / -1;
        justify-content: flex-start;
        margin-top: 0.5rem;
      }
    }
</style>
