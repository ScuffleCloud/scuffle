<script lang="ts">
    import InlineNotification from "$lib/components/inline-notification.svelte";
    import LoadingSpinner from "$lib/components/loading-spinner.svelte";
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
        <LoadingSpinner />
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
</style>
