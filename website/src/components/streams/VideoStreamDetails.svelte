<script lang="ts">
    import type { Streamed } from '$lib/types';
    import { Tabs } from 'melt/builders';
    import type { VideoStream } from './types';
    import StreamDetailsHeader from './StreamDetailsHeader.svelte';

    type Props = {
        streamedData: Streamed<VideoStream>;
    };

    const { streamedData }: Props = $props();

    function formatCreationTime(timestamp: string) {
        const created = new Date(timestamp);
        const now = new Date();
        const diffMinutes = Math.floor((now.getTime() - created.getTime()) / (1000 * 60));
        return `${diffMinutes} minutes ago`;
    }

    // Initialize tabs
    const tabs = new Tabs({
        value: 'overview',
        selectWhenFocused: true,
        orientation: 'horizontal',
    });

    const tabIds = ['overview', 'events', 'assets'];
</script>

<!-- Vibe coded. Just placeholder -->
<div class="stream-page">
    {#await streamedData}
        <!-- TODO: Add skeleton loading state -->
        <div class="loading">Loading...</div>
    {:then stream}
        <div class="stream-container">
            <StreamDetailsHeader {stream} />
        </div>
    {:catch error}
        <div class="error">Error: {error.message}</div>
    {/await}
</div>

<style>
    .stream-page {
        width: 100%;
        max-width: 1200px;
        margin: 0 auto;
        padding: 20px;

        .loading,
        .error {
            display: flex;
            justify-content: center;
            align-items: center;
            height: 400px;
            font-size: 18px;
            color: #666;
        }

        .error {
            color: #e53935;
        }

        .stream-container {
            width: 100%;
        }
    }
</style>
