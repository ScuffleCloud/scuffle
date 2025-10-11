<!-- components/badge.svelte -->
<script lang="ts">
    import type { Snippet } from "svelte";

    export type BadgeVariant =
        | "enabled"
        | "disabled"
        | "warning"
        | "info";

    interface Props {
        variant?: BadgeVariant;
        children: Snippet;
    }

    const { variant = "info", children }: Props = $props();

    function getVariantClass(variant: BadgeVariant): string {
        switch (variant) {
            case "enabled":
                return "badge-enabled";
            case "disabled":
                return "badge-disabled";
            case "warning":
                return "badge-warning";
            case "info":
                return "badge-info";
        }
    }
</script>

<span class="badge {getVariantClass(variant)}">
    {@render children()}
</span>

<style>
    .badge {
      font-size: 0.875rem;
      font-weight: 700;
      line-height: 1rem;
      border-radius: 5.25rem;
      padding: 0.25rem 0.5625rem;
      display: inline-flex;
      align-items: center;
    }

    .badge-enabled {
      background: #dcfce7;
      color: #16a34a;
    }

    .badge-disabled {
      background: var(--colors-gray90);
      color: var(--colors-gray90);
    }

    .badge-warning {
      background: #fef3c7;
      color: #d97706;
    }

    .badge-info {
      background: var(--colors-gray40);
      color: var(--colors-gray90);
    }
</style>
