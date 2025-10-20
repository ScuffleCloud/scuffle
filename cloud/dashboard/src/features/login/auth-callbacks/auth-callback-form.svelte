<script lang="ts">
    import InlineNotification from "$lib/components/inline-notification.svelte";
    import type { CreateMutationResult } from "@tanstack/svelte-query";

    type Props = {
        message: string;
        mutation: Pick<
            CreateMutationResult<unknown, Error, unknown, unknown>,
            "error" | "isPending"
        >;
    };

    let { message, mutation }: Props = $props();
</script>

<!-- TBD if we want something like this. Could also just use the same loading style we decide on for our app and ignore the type of callback -->
<div class="content-container">
    <div>{message}</div>
    {#if mutation.isPending}
        <div class="spinner"></div>
    {/if}
    {#if mutation.error}
        <InlineNotification
            type="error"
            message={mutation.error.message || "An error occurred"}
        />
    {/if}
</div>

<style>
    .content-container {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 0.5rem;
    }

    .spinner {
      width: 1rem;
      height: 1rem;
      border: 2px solid rgba(255, 255, 255, 0.3);
      border-top: 2px solid black;
      border-radius: 50%;
      animation: spin 1s linear infinite;
    }

    @keyframes spin {
      to {
        transform: rotate(360deg);
      }
    }
</style>
