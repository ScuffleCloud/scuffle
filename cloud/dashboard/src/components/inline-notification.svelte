<script lang="ts">
    import IconAlarm from "$lib/images/icon-alarm.svelte";
    import IconCheckSmall from "$lib/images/icon-check-small.svelte";
    import IconInfo from "$lib/images/icon-info.svelte";

    type NotificationType =
        | "error"
        | "warning"
        | "success"
        | "info"
        | "neutral";

    interface Props {
        type?: NotificationType;
        message: string;
    }

    let {
        type = "info",
        message,
    }: Props = $props();

    let isVisible = $state(true);

    const iconComponents: Record<NotificationType, any> = {
        error: IconAlarm,
        warning: IconInfo,
        success: IconCheckSmall,
        info: IconInfo,
        neutral: IconInfo,
    };

    const IconComponent = $derived(iconComponents[type]);
</script>

{#if isVisible}
    <div class="notification notification-{type}">
        <div class="notification-icon">
            <IconComponent />
        </div>

        <div class="notification-message">
            {message}
        </div>
    </div>
{/if}

<style>
    .notification {
      display: flex;
      align-items: center;
      gap: 1rem;
      padding: 1rem;
      border-radius: 0.5rem;
      transition: all 0.2s ease;
      width: 100%;
    }

    .notification-icon {
      flex-shrink: 0;
      display: flex;
      align-items: center;
      justify-content: center;
      color: white;
    }

    .notification-message {
      font-weight: 500;
      font-size: 1rem;
    }

    .notification-close {
      flex-shrink: 0;
      background: none;
      border: none;
      cursor: pointer;
      padding: 0;
      width: 1.5rem;
      height: 1.5rem;
      display: flex;
      align-items: center;
      justify-content: center;
      transition: opacity 0.2s ease;
    }

    .notification-close:hover {
      opacity: 0.7;
    }

    .notification-error {
      background-color: var(--colors-red10);
      color: var(--colors-red70);
    }
    .notification-warning {
      background-color: #fef3c7;
      border-color: #fde68a;
      color: #78350f;
    }

    .notification-warning .notification-icon {
      background-color: #ca8a04;
    }

    /* Success variant */
    .notification-success {
      background-color: #d1fae5;
      border-color: #a7f3d0;
      color: #14532d;
    }

    .notification-success .notification-icon {
      background-color: #16a34a;
    }

    /* Info variant */
    .notification-info {
      background-color: #dbeafe;
      border-color: #bfdbfe;
      color: #1e3a8a;
    }

    .notification-info .notification-icon {
      background-color: #2563eb;
    }

    /* Neutral variant */
    .notification-neutral {
      background-color: #f3f4f6;
      border-color: #d1d5db;
      color: #1f2937;
    }

    .notification-neutral .notification-icon {
      background-color: #4b5563;
    }
</style>
