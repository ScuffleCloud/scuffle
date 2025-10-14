<script lang="ts">
    import type { Component, Snippet } from "svelte";

    interface Props {
        title: string;
        subtitle?: string;
        icon: Component;
        children: Snippet;
    }

    const { title, subtitle, icon: Icon, children }: Props = $props();
</script>

<div class="settings-block">
    <div class="block-header">
        <div class="block-icon">
            <Icon />
        </div>
        <div class="block-title-content">
            <h3 class="block-title">{title}</h3>
            {#if subtitle}
                <span class="block-subtitle">{subtitle}</span>
            {/if}
        </div>
    </div>
    <div class="cards-container">
        {@render children()}
    </div>
</div>

<style>
    .settings-block {
      background: var(--colors-gray50);
      border-radius: 8px;
      padding: 0.375rem;
      max-width: 100%;
    }

    .block-header {
      display: flex;
      align-items: center;

      .block-icon {
        display: flex;
        padding: 0.5rem;
      }

      .block-title-content {
        display: flex;
        align-items: center;
        gap: 0.5rem;

        .block-title {
          font-size: 1rem;
          font-weight: 700;
          color: var(--colors-brown90);
          line-height: 1.5rem;
          margin: 0;
        }

        .block-subtitle {
          font-size: 1rem;
          font-weight: 500;
          color: var(--colors-brown50);
          line-height: 1.5rem;
        }
      }
    }

    .cards-container {
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
    }
</style>
