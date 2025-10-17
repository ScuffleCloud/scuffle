<script lang="ts">
    import IconDots from "$lib/images/icon-dots.svelte";
    import { createDropdownMenu, melt } from "@melt-ui/svelte";
    import type { Component } from "svelte";

    interface MenuItem {
        label: string;
        key: string;
        icon: Component;
        onClick: () => void;
        variant?: "default" | "danger";
    }

    interface Props {
        items: MenuItem[];
        triggerIcon?: Component;
        triggerLabel?: string;
    }

    let { items, triggerIcon: TriggerIcon, triggerLabel = "Actions" }:
        Props = $props();

    const {
        elements: { trigger, menu, item },
        states: { open },
    } = createDropdownMenu({
        forceVisible: true,
        positioning: {
            placement: "bottom-end",
        },
    });
</script>

<div class="actions-menu-container">
    <button
        class="icon-button"
        use:melt={$trigger}
        aria-label={triggerLabel}
    >
        {#if TriggerIcon}
            <TriggerIcon />
        {:else}
            <IconDots />
        {/if}
    </button>

    {#if $open}
        <div class="dropdown-menu" use:melt={$menu}>
            {#each items as menuItem (menuItem.key)}
                <button
                    class="menu-item"
                    class:danger={menuItem.variant === "danger"}
                    use:melt={$item}
                    onclick={menuItem.onClick}
                >
                    <menuItem.icon />
                    <span>{menuItem.label}</span>
                </button>
            {/each}
        </div>
    {/if}
</div>

<style>
    .actions-menu-container {
      position: relative;
      display: inline-block;
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
      border-radius: 0.25rem;
    }

    .icon-button:hover {
      color: var(--colors-brown90);
      background-color: var(--colors-gray40);
    }

    .dropdown-menu {
      z-index: 50;
      background: #ffffff;
      border: 1px solid #e5e5e5;
      border-radius: 0.5rem;
      padding: 0.25rem;
      min-width: 160px;
      animation: fadeIn 0.15s ease-out;
    }

    .menu-item {
      width: 100%;
      display: flex;
      align-items: center;
      gap: 0.75rem;
      padding: 0.625rem 0.75rem;
      background: none;
      border: none;
      cursor: pointer;
      border-radius: 0.375rem;
      transition: background-color 0.15s ease;
      font-size: 0.875rem;
      font-weight: 500;
      color: #374151;
      text-align: left;
    }

    .menu-item:hover {
      background-color: #f3f4f6;
    }

    .menu-item.danger {
      color: #dc2626;
    }

    .menu-item.danger:hover {
      background-color: #fee;
    }

    @keyframes fadeIn {
      from {
        opacity: 0;
        transform: translateY(-4px);
      }
      to {
        opacity: 1;
        transform: translateY(0);
      }
    }
</style>
