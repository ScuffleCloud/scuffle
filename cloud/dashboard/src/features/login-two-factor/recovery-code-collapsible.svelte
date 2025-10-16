<script lang="ts">
    import IconArrowDown from "$lib/images/icon-arrow-down.svelte";
    import IconArrowUp from "$lib/images/icon-arrow-up.svelte";
    import { Collapsible } from "melt/builders";

    interface Props {
        onAction: () => void;
    }

    let {
        onAction,
    }: Props = $props();

    const troubleshootCollapsible = new Collapsible();
</script>

<div class="troubleshoot-section">
    <button
        type="button"
        class="troubleshoot-trigger"
        {...troubleshootCollapsible.trigger}
    >
        {#if troubleshootCollapsible.open}
            <IconArrowUp />
        {:else}
            <IconArrowDown />
        {/if}
        Having issues with authentication?
    </button>

    {#if troubleshootCollapsible.open}
        <div {...troubleshootCollapsible.content} class="troubleshoot-content">
            <button
                type="button"
                onclick={onAction}
                class="action-btn"
            >
                Use a backup code
            </button>
        </div>
    {/if}
</div>

<style>
    .troubleshoot-section {
      margin-top: 1rem;
      width: 100%;
    }

    .troubleshoot-trigger {
      width: 100%;
      background: none;
      border: none;
      padding: 0.75rem 0;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
      color: #6b7280;
      font-size: 1rem;
    }

    .troubleshoot-trigger:hover {
      color: #374151;
    }

    .chevron {
      transition: transform 0.2s ease;
      flex-shrink: 0;
    }

    .chevron.open {
      transform: rotate(180deg);
    }

    .troubleshoot-content {
      padding-top: 0.5rem;
    }

    .action-btn {
      width: 100%;
      padding: 0.75rem;
      background: white;
      color: #374151;
      border: 1px solid #d1d5db;
      cursor: pointer;
      margin-bottom: 0.5rem;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
      border-radius: 0.5rem;
    }

    .action-btn:hover:not(:disabled) {
      background: #f9fafb;
      border-color: #9ca3af;
    }

    .action-btn:disabled {
      background: white;
      color: #9ca3af;
      cursor: not-allowed;
    }

    .troubleshoot-items {
      color: #6b7280;
      font-size: 0.875rem;
      line-height: 1.5;
    }
</style>
