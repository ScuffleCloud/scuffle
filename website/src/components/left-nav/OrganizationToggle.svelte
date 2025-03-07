<script lang="ts">
    import IconSwitch from '$lib/images/IconSwitch.svelte';
    import { userStore } from '$lib/stores/userStore.svelte';

    // Props for organization info

    // we should get this information from a generic hook somewhere
    // const {
    //     name = 'Personal',
    //     username = 'username',
    //     avatarColor = '#FFCC80', // Light orange color from the screenshot
    // } = $props();

    // Not sure why this isn't working
    const user = $derived(userStore.user);
    // const user = userStore.user;

    let expanded = $state(true);

    // Toggle expanded state
    function toggleExpanded() {
        expanded = !expanded;
    }

    function handleKeyDown(event: KeyboardEvent) {
        if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault();
            toggleExpanded();
        }
    }
</script>

<div class="organization-info">
    <button
        type="button"
        class="org-header"
        onclick={toggleExpanded}
        onkeydown={handleKeyDown}
        aria-expanded={expanded}
    >
        <!-- <div class="avatar" style:background-color={user?.organizations[0].avatar}></div> -->
        <div class="avatar" style:background-color={'#FFCC80'}></div>
        <div class="org-details">
            <div class="org-name">{user?.organizations[0].name}</div>
            <div class="org-username">{user?.email}</div>
        </div>
        <IconSwitch />
    </button>

    {#if expanded}
        <div class="org-content">
            <!-- This should loop over all organizations that aren't currently selected and display them -->
            <p>Organization content</p>
            {#if user?.organizations}
                <ul>
                    {#each user?.organizations as org}
                        <li>{org.name}</li>
                    {/each}
                </ul>
            {/if}
        </div>
    {/if}
</div>

<style>
    .organization-info {
        background-color: #f9f5f5;
        margin-bottom: 1rem;
        padding: 0.5rem 0.75rem;

        .org-header {
            display: flex;
            align-items: center;
            cursor: pointer;
            gap: 0.5rem;
            width: 100%;

            .avatar {
                width: 2rem;
                height: 2rem;
                border-radius: 0.5rem;

                flex-shrink: 0;
            }

            .org-details {
                flex: 1;
                /* align left  */
                text-align: left;
                min-width: 0;

                .org-name {
                    color: var(--color-brown-600);
                    font-size: 0.875rem;
                    font-weight: 600;
                    line-height: normal;
                }

                .org-username {
                    color: var(--color-brown700);
                    font-size: 1rem;
                    font-weight: 700;
                    line-height: normal;
                    text-overflow: ellipsis;
                    overflow: hidden;
                    white-space: nowrap;
                }
            }
        }

        .org-content {
            margin-top: 1rem;
            padding-top: 1rem;
            border-top: 1px solid rgba(0, 0, 0, 0.1);
        }
    }
</style>
