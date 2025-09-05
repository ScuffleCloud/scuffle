<script lang="ts">
    import Header from "./header.svelte";
    import StreamsTable from "./streams-table.svelte";

    import SearchInput from "$components/search-input.svelte";
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
<SearchInput bind:value={searchQuery} placeholder="Search..." />
{#await streamedData}
    <div>Loading...</div>
{:then resolvedStreams}
    <StreamsTable streams={resolvedStreams.results} />
{:catch error}
    <p>{error.message}</p>
{/await}
