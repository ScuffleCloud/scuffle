<script lang="ts">
    import SearchInput from "$components/search-input.svelte";
    import AssetCard from "./asset-card.svelte";
    import Header from "./assets-header.svelte";
    import SlidingToggle from "./sliding-toggle.svelte";

    // Search functionality
    let searchQuery = $state("");
    let displayMode = $state(1); // 1 = grid, 2 = list

    // These will come from a paginated response
    const streams = [
        {
            status: "Live",
            streamName: "unique-stream-name",
            streamId: "8a28e499d6s7987fd981293fd981293",
            startTime: "May 5, 04:01:11 PM EDT",
            duration: "1h 30min",
            endTime: "May 6, 04:01:11 PM EDT",
        },
        {
            status: "Live",
            streamName: "unique-stream-name",
            streamId: "8a28e499d6s7987fd981293fd981293",
            startTime: "May 5, 04:01:11 PM EDT",
            duration: "1h 30min",
            endTime: "May 6, 04:01:11 PM EDT",
        },
        {
            status: "Finished",
            streamName: "unique-stream-name",
            streamId: "8a28e499d6s7987fd981293fd981293",
            startTime: "May 5, 04:01:11 PM EDT",
            duration: "1h 30min",
            endTime: "May 6, 04:01:11 PM EDT",
        },
        {
            status: "Finished",
            streamName: "unique-stream-name",
            streamId: "8a28e499d6s7987fd981293fd981293",
            startTime: "May 5, 04:01:11 PM EDT",
            duration: "1h 30min",
            endTime: "May 6, 04:01:11 PM EDT",
        },
    ];

    function handleDisplayModeToggle(value: number) {
        displayMode = value;
    }
</script>

<Header />
<div class="row-2">
    <SearchInput bind:value={searchQuery} placeholder="Search..." />
    <SlidingToggle
        leftLabel="Light"
        rightLabel="Dark"
        value={displayMode.toString()}
        onToggle={handleDisplayModeToggle}
    />
</div>
<div class="card-container">
    {#each streams as stream, i}
        <AssetCard {...stream} />
    {/each}
</div>

<style>
    .card-container {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }

    .row-2 {
      display: flex;
      justify-content: space-between;
      margin-bottom: 1.5rem;
      gap: 0.5rem;
    }
</style>
