<script lang="ts">
    import Header from './Header.svelte';
    import StreamsTable from './StreamsTable.svelte';

    import type { Streamed } from '$lib/types';
    // Define types
    type VideoStatus = 'Live' | 'Finished';

    interface VideoStream {
        id: string;
        name: string;
        status: VideoStatus;
        created: string;
    }

    const { streamedData } = $props<{
        streamedData: Streamed<VideoStream[]>;
    }>();

    let streams: VideoStream[] = $state([
        {
            id: '8a28e4dbd6...',
            name: 'purple-angry-bottle...',
            status: 'Live',
            created: '2min ago',
        },
        {
            id: '8a28e499d61...',
            name: 'orange-fluffy-chair...',
            status: 'Live',
            created: '1.2.2025',
        },
        {
            id: '8a28e4dbd62...',
            name: 'purple-angry-bottle...',
            status: 'Finished',
            created: '1.1.2025',
        },
        {
            id: '8a28e4dbd63...',
            name: 'purple-angry-bottle...',
            status: 'Finished',
            created: '1.1.2025',
        },
    ]);

    // Search functionality
    let searchQuery = $state('');
</script>

<Header />
{#await streamedData}
    Loading streams...dwq
{:then resolvedStreams}
    <pre>{JSON.stringify(resolvedStreams, null, 2)}</pre>
{:catch error}
    <p>error loading streams: {error.message}</p>
{/await}
<div class="search-container">
    <div class="search-input">
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <circle cx="11" cy="11" r="8"></circle>
            <path d="m21 21-4.3-4.3"></path>
        </svg>
        <input type="text" placeholder="Search..." bind:value={searchQuery} />
    </div>
</div>
<StreamsTable {streams} />

<style>
    .search-container {
        display: flex;
        justify-content: space-between;
        margin-bottom: 1.5rem;

        .search-input {
            position: relative;
            flex: 1;
            max-width: 600px;

            svg {
                position: absolute;
                left: 1rem;
                top: 50%;
                transform: translateY(-50%);
                color: #888;
            }

            input {
                width: 100%;
                padding: 0.75rem 1rem 0.75rem 2.5rem;
                border: none;
                border-radius: 2rem;
                background-color: #f5f5f5;
                font-size: 1rem;

                &:focus {
                    outline: none;
                    box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.1);
                }
            }
        }
    }
</style>
