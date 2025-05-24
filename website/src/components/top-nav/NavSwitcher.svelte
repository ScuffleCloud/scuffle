<script lang="ts">
    import { createDropdownMenu, melt } from '@melt-ui/svelte';
    import IconSwitch from '$lib/images/IconSwitch.svelte';

    interface MenuItem {
        id: string;
        name: string;
        imageUrl?: string;
    }

    interface Props {
        name: string;
        imageUrl?: string;
        items: MenuItem[];
        onClick: (id: string) => void;
    }

    const { name, imageUrl, items, onClick }: Props = $props();

    const {
        elements: { trigger, menu, item },
    } = createDropdownMenu();
</script>

<div>
    <button class="nav-switcher" use:melt={$trigger}>
        <div class="image-container">
            {#if imageUrl}
                <img src={imageUrl} alt={name ?? 'Organization logo'} class="organization-image" />
            {/if}
            {name}
        </div>
        <IconSwitch />
    </button>
    <div class="dropdown-menu" use:melt={$menu}>
        {#each items as menuItem}
            <div class="menu-item" use:melt={$item} onclick={() => onClick(menuItem.id)}>
                <div class="menu-item-content">
                    {#if menuItem.imageUrl}
                        <img
                            src={menuItem.imageUrl}
                            alt={`${menuItem.name} logo`}
                            class="menu-item-image"
                        />
                    {/if}
                    <span class="menu-item-name">{menuItem.name}</span>
                </div>
            </div>
        {/each}
    </div>
</div>

<style>
    .nav-switcher {
        display: flex;
        align-items: center;
        justify-content: space-between;
        width: 100%;
        color: #201617;
        font-size: 1rem;
        font-style: normal;
        font-weight: 500;
        line-height: 1.5rem;
        padding: 0.38rem;
        background: transparent;
        border: none;
        cursor: pointer;

        &:hover {
            background-color: rgba(0, 0, 0, 0.05);
        }

        .image-container {
            display: flex;
            align-items: center;
            gap: 0.5rem;

            .organization-image {
                width: 1.5rem;
                height: 1.5rem;
                object-fit: cover;
                border-radius: 0.25rem;
            }
        }
    }

    .dropdown-menu {
        z-index: 50;
        min-width: 220px;
        background: white;
        border-radius: 0.5rem;
        padding: 0.5rem;
        box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);

        .menu-item {
            padding: 0.5rem;
            cursor: pointer;
            border-radius: 0.25rem;

            &:hover {
                background-color: rgba(0, 0, 0, 0.05);
            }

            .menu-item-content {
                display: flex;
                align-items: center;
                gap: 0.5rem;

                .menu-item-image {
                    width: 1.5rem;
                    height: 1.5rem;
                    object-fit: cover;
                    border-radius: 0.25rem;
                }

                .menu-item-name {
                    font-size: 0.875rem;
                    color: #201617;
                }
            }
        }
    }
</style>
