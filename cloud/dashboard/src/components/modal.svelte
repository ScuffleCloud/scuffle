<script lang="ts">
    import IconArrowLeft from "$lib/images/icon-arrow-left.svelte";
    import IconCloseX from "$lib/images/icon-close-x.svelte";
    import { createDialog, melt } from "@melt-ui/svelte";
    import type { Snippet } from "svelte";
    import { fade, fly } from "svelte/transition";

    interface Props {
        triggerLabel?: string;
        triggerClass?: string;
        title?: string;
        onBack?: () => void;
        onClose?: () => void;
        hideCloseButton?: boolean;
        children: Snippet;
    }

    let {
        triggerLabel = "Open",
        triggerClass = "",
        title,
        onBack,
        onClose,
        hideCloseButton = false,
        children,
    }: Props = $props();

    const {
        elements: { trigger, overlay, content, close, portalled },
        states: { open },
    } = createDialog({
        forceVisible: true,
        onOpenChange: ({ next }) => {
            if (!next && onClose) {
                onClose();
            }
            return next;
        },
    });

    export function closeDialog() {
        open.set(false);
    }
</script>

<button use:melt={$trigger} class={triggerClass || "default-trigger"}>
    {triggerLabel}
</button>

{#if $open}
    <div use:melt={$portalled}>
        <div
            use:melt={$overlay}
            class="overlay"
            transition:fade={{ duration: 150 }}
        >
        </div>

        <div
            use:melt={$content}
            class="dialog-content"
            transition:fly={{ duration: 150, y: 8 }}
        >
            <div class="dialog-header">
                {#if onBack}
                    <button
                        class="back-button"
                        onclick={onBack}
                        aria-label="go back"
                    >
                        <IconArrowLeft />
                    </button>
                {/if}

                {#if title}
                    <h2 class="dialog-title">{title}</h2>
                {/if}

                {#if !hideCloseButton}
                    <button
                        use:melt={$close}
                        aria-label="close"
                        class="close-button"
                    >
                        <IconCloseX />
                    </button>
                {/if}
            </div>

            <div class="dialog-body">
                {@render children()}
            </div>
        </div>
    </div>
{/if}

<style>
    .default-trigger {
      padding: 0.75rem 1.5rem;
      border-radius: 0.5rem;
      background-color: rgb(247, 177, 85);
      color: rgb(65, 28, 9);
      font-weight: 600;
      border: none;
      cursor: pointer;
      transition: opacity 0.2s;
    }

    .default-trigger:hover {
      opacity: 0.9;
    }

    .overlay {
      position: fixed;
      inset: 0;
      z-index: 40;
      background-color: rgba(0, 0, 0, 0.5);
    }

    .dialog-content {
      position: fixed;
      left: 50%;
      top: 50%;
      transform: translate(-50%, -50%);
      z-index: 50;
      width: 90vw;
      max-width: 500px;
      background-color: rgb(245, 242, 238);
      border-radius: 1.5rem;
      padding: 2rem;
      box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
    }

    .dialog-header {
      position: relative;
      display: flex;
      align-items: center;
      justify-content: center;
      margin-bottom: 1.5rem;
      min-height: 2rem;
    }

    .back-button {
      position: absolute;
      left: 0;
      top: 50%;
      transform: translateY(-50%);
      display: flex;
      align-items: center;
      justify-content: center;
      width: 2rem;
      height: 2rem;
      background: none;
      border: none;
      cursor: pointer;
      color: rgb(82, 82, 82);
      border-radius: 0.375rem;
    }

    .back-button:hover {
      background-color: rgba(0, 0, 0, 0.05);
    }

    .dialog-title {
      font-size: 1.5rem;
      font-weight: 700;
      line-height: 2rem;
      color: rgb(23, 23, 23);
      margin: 0;
      text-align: center;
    }

    .close-button {
      position: absolute;
      right: 0;
      top: 50%;
      transform: translateY(-50%);
      display: flex;
      align-items: center;
      justify-content: center;
      width: 2rem;
      height: 2rem;
      background: none;
      border: none;
      cursor: pointer;
      color: rgb(38, 38, 38);
      border-radius: 0.375rem;
    }

    .close-button:hover {
      background-color: rgba(0, 0, 0, 0.05);
    }
</style>
