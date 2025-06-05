<script lang="ts">
    import IconSwitch from '$lib/images/IconSwitch.svelte';
    import { userStore } from '$lib/stores/userStore.svelte';
    import OrganizationDropdown from './OrganizationDropdown.svelte';
    import { createPopover, melt } from '@melt-ui/svelte';

    type Props = {
        isCollapsed?: boolean;
    };

    const { isCollapsed = false }: Props = $props();

    const {
        elements: { trigger, content },
        states: { open },
    } = createPopover({
        preventScroll: true,
    });

    const user = $derived(userStore.user);
</script>

<div class="organization-info" class:collapsed={isCollapsed}>
    <button type="button" class="org-header-button" use:melt={$trigger}>
        <div class="avatar" style:background-color={'#FFCC80'}></div>
        {#if !isCollapsed}
            <div class="org-details">
                <div class="org-name">{user?.organizations[0].name}</div>
                <div class="org-username">{user?.email}</div>
            </div>
            <IconSwitch />
        {/if}
    </button>
    {#if $open}
        <div use:melt={$content} class="popover-content">
            <OrganizationDropdown organizations={user?.organizations} />
        </div>
    {/if}
</div>

<style>
    .organization-info {
        margin-bottom: 1rem;
        padding: 0;

        .org-header-button {
            display: flex;
            align-items: center;
            cursor: pointer;
            gap: 0.5rem;
            width: 100%;
            padding: 0.5rem 0.56rem;
            border: none;
            background-color: transparent;
            transition: justify-content 0.3s ease;

            &:hover {
                background-color: rgba(0, 0, 0, 0.05);
            }

            .avatar {
                width: 2rem;
                height: 2rem;
                border-radius: 0.5rem;
                flex-shrink: 0;
            }

            .org-details {
                flex: 1;
                text-align: left;
                min-width: 0;
                text-wrap: nowrap;
                transition: opacity 0.2s ease;

                .org-name {
                    color: var(--colors-brown-600);
                    font-size: 0.875rem;
                    font-weight: 600;
                    line-height: normal;
                }

                .org-username {
                    color: var(--colors-brown-700);
                    font-size: 1rem;
                    font-weight: 700;
                    line-height: normal;
                    text-overflow: ellipsis;
                    overflow: hidden;
                    white-space: nowrap;
                }
            }
        }

        &.collapsed .org-header-button {
            justify-content: center;
            padding: 0.5rem;
        }
    }

    .popover-content {
        z-index: 5;
        border-radius: 0.5rem;
        width: 15rem;
        max-width: 100%;
    }
</style>
