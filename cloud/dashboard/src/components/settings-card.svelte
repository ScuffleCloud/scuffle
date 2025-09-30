<script lang="ts">
    import type { Snippet } from "svelte";

    interface Props {
        title: string;
        status?: {
            label: string;
            variant: "enabled" | "disabled" | "warning";
        };
        description?: string;
        children?: Snippet;
    }

    const { title, status, description, children }: Props = $props();

    function getStatusClass(
        variant: "enabled" | "disabled" | "warning",
    ) {
        switch (variant) {
            case "enabled":
                return "status-enabled";
            case "disabled":
                return "status-disabled";
            case "warning":
                return "status-warning";
        }
    }
</script>

<div class="card">
    <div class="card-header">
        <div class="card-title-row">
            <h4 class="card-title">{title}</h4>
            {#if status}
                <span class="status-badge {getStatusClass(status.variant)}">
                    {status.label}
                </span>
            {/if}
        </div>
    </div>

    {#if description}
        <p class="card-description">{description}</p>
    {/if}

    {#if children}
        {@render children()}
    {/if}
</div>

<style>
    .card {
      background: var(--colors-gray20);
      border-radius: 0.5rem;
      padding: 1rem;
      border: 1px solid var(--colors-teal30);

      .card-header {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 0.25rem;
        margin-bottom: 0.75rem;
      }
    }

    .card-title-row {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      width: 100%;
    }

    .card-title {
      color: var(--colors-brown90);
      font-size: 1.125rem;
      font-weight: 700;
      line-height: 1.5rem;
      margin: 0;
    }

    .status-badge {
      color: var(--colors-gray90);
      font-size: 0.875rem;
      font-weight: 700;
      line-height: 1rem;
      border-radius: 5.25rem;
      padding: 0.25rem 0.5625rem;
      background: var(--colors-gray50);
    }

    .status-enabled {
      background: #dcfce7;
      color: #16a34a;
    }

    .status-disabled {
      background: var(--colors-gray50);
    }

    .status-warning {
      background: #fef3c7;
      color: #d97706;
    }

    .card-description {
      margin-bottom: 0.75rem;
      color: var(--colors-brown90);
      font-size: 1rem;
      font-weight: 500;
      line-height: 1.5rem;
    }
</style>
