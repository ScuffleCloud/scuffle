<script lang="ts">
    import IconArrowLeft from "$lib/images/icon-arrow-left.svelte";

    interface Props {
        onBack?: () => void;
        title: string;
    }

    let { onBack, title }: Props = $props();
</script>

<div class="header">
    {#if onBack}
        <button type="button" onclick={onBack} class="back-button">
            <IconArrowLeft />
        </button>
    {/if}
    <h1 class="title">{title}</h1>
</div>

<style>
    .header {
      display: flex;
      align-items: center;
      position: relative;
      margin-bottom: 1.25rem;
    }

    .back-button {
      background: none;
      border: none;
      color: #6b7280;
      cursor: pointer;
      font-size: 0.875rem;
      padding: 0;
      position: absolute;
      left: 0;
      top: 50%;
      transform: translateY(-50%);
      display: flex;
      align-items: center;
      gap: 0.25rem;
    }

    .back-button:hover {
      color: #374151;
    }

    .title {
      font-size: 1.5rem;
      font-weight: 600;
      margin: 0 auto;
      text-align: center;
      flex: 1;
    }
</style>
