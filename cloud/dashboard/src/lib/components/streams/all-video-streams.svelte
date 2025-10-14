<script lang="ts">
    import Header from "./header.svelte";
    import StreamsTable from "./streams-table.svelte";

    import SearchInput from "$lib/components/search-input.svelte";
    import type { ListResponse, Streamed } from "$lib/types";
    import type { VideoStream } from "./types";

    type Props = {
        streamedData: Streamed<ListResponse<VideoStream>>;
    };

    const { streamedData }: Props = $props();

    // Search functionality
    let searchQuery = $state("");
</script>

<Header />
<div class="search-row">
    <SearchInput bind:value={searchQuery} placeholder="Search..." />
</div>
{#await streamedData}
    <div>Loading...</div>
{:then resolvedStreams}
    <StreamsTable streams={resolvedStreams.results} />
{:catch error}
    <p>{error.message}</p>
{/await}

<style>
    .search-row {
      display: flex;
      justify-content: space-between;
      margin-bottom: 1.5rem;
      gap: 0.5rem;
      height: 2.5rem;
    }
</style>
