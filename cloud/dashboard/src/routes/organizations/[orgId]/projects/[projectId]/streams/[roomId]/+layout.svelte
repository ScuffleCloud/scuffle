<script lang="ts">
    import { page } from "$app/state";
    import StreamDetailsHeader from "$components/streams/stream-detail-header.svelte";
    import type { VideoStream } from "$components/streams/types";
    import Tabs from "$components/tabs.svelte";
    import IconAssets_2 from "$lib/images/icon-assets-2.svelte";
    import IconEvents from "$lib/images/icon-events.svelte";
    import IconPuzzle from "$lib/images/icon-puzzle.svelte";
    import IconSettings2 from "$lib/images/icon-settings2.svelte";
    import type { Streamed } from "$lib/types";
    import type { Snippet } from "svelte";
    import type { LayoutData } from "./$types";

    type ChildProps = {
        data: VideoStream;
    };

    type Props = {
        data: LayoutData;
        children: Snippet<[ChildProps]>;
    };

    const { data: pageData, children }: Props = $props();

    const tabs = [
        { id: "overview", label: "Overview", icon: IconPuzzle },
        { id: "events", label: "Events", icon: IconEvents },
        { id: "assets", label: "Assets", icon: IconAssets_2 },
        { id: "settings", label: "Settings", icon: IconSettings2 },
    ];
    const baseUrl =
        `/organizations/${page.params.orgId}/projects/${page.params.projectId}/streams/${page.params.roomId}/`;
</script>

<div class="page-bg">
    {#await pageData.stream}
        <div class="loading">Loading...</div>
    {:then stream}
        <StreamDetailsHeader {stream} />
        <Tabs {tabs} {baseUrl} defaultTab="overview">
            {@render children({ data: stream })}
        </Tabs>
    {:catch error}
        <div class="error">Error: {error.message}</div>
    {/await}
</div>

<style>
    .page-bg {
      background-color: var(--colors-light100);
      margin: 0 auto;
      width: 100%;
      max-width: 1200px;
      padding: 2rem;
    }

    .loading {
      display: flex;
      justify-content: center;
      align-items: center;
      padding: 2rem;
      color: var(--colors-yellow90);
    }

    .error {
      display: flex;
      justify-content: center;
      align-items: center;
      padding: 2rem;
      color: #ef4444;
    }
</style>
