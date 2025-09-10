<script lang="ts">
    import SearchInput from "$components/search-input.svelte";
    import IconLayout_3 from "$lib/images/icon-layout-3.svelte";
    import IconLayout from "$lib/images/icon-layout.svelte";
    import AssetCard from "./asset-card.svelte";
    import Header from "./assets-header.svelte";
    import SlidingToggle from "./sliding-toggle.svelte";
    import TimeFilterDropdown from "./time-filter-dropdown.svelte";
    import { DISPLAY_MODES } from "./types";

    // Search functionality
    let searchQuery = $state("");
    let displayMode = $state(DISPLAY_MODES.GRID);

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

    function handleDisplayModeToggle() {
        if (displayMode === DISPLAY_MODES.GRID) {
            displayMode = DISPLAY_MODES.LIST;
        } else {
            displayMode = DISPLAY_MODES.GRID;
        }
    }

    let selectedTimeFilter = $state("latest");

    function handleTimeFilterChange(filterId: string) {
        selectedTimeFilter = filterId;
    }
</script>

<Header />
<div class="search-row">
    <SearchInput bind:value={searchQuery} placeholder="Search..." />
    <TimeFilterDropdown
        selectedFilter={selectedTimeFilter}
        onFilterChange={handleTimeFilterChange}
    />
    <SlidingToggle
        leftLabel={IconLayout}
        rightLabel={IconLayout_3}
        value={displayMode === DISPLAY_MODES.GRID ? "left" : "right"}
        onToggle={handleDisplayModeToggle}
    />
</div>
<div class="separator"></div>
<div class="card-container">
    {#each streams as stream, i}
        <AssetCard {...stream} />
    {/each}
</div>

<div class="separator"></div>

<style>
    .card-container {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }

    .separator {
      height: 1px;
      background-color: var(--colors-gray50);
      margin: 1.5rem 0;
    }

    .search-row {
      display: flex;
      justify-content: space-between;
      margin-bottom: 1.5rem;
      gap: 0.5rem;
      height: 2.5rem;
    }
</style>
