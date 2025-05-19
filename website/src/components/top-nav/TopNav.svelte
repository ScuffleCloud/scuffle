<script lang="ts">
    import Search from '$lib/images/Search.svelte';
    import { onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    // TODO: Configure routing from root
    import { page } from '$app/state';

    let showSearchModal = $state(false);
    let searchInput = $state<HTMLInputElement | null>(null);

    // Handle keyboard shortcut
    function handleKeydown(event: KeyboardEvent) {
        if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
            event.preventDefault();
            showSearchModal = true;
        }

        if (event.key === 'Escape' && showSearchModal) {
            showSearchModal = false;
        }
    }

    $effect(() => {
        if (showSearchModal && searchInput) {
            setTimeout(() => {
                if (searchInput) searchInput.focus();
            }, 50);
        }
    });

    // Add and remove event listener
    onMount(() => {
        window.addEventListener('keydown', handleKeydown);
        return () => {
            window.removeEventListener('keydown', handleKeydown);
        };
    });

    function closeModal() {
        showSearchModal = false;
    }

    let breadcrumbs = $derived(
        page.url.pathname
            .split('/')
            .filter(Boolean)
            .map((segment, index, segments) => ({
                label: segment.replace(/[-_]/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase()),
                href: '/' + segments.slice(0, index + 1).join('/'),
            })),
    );
</script>

<header class="top-nav">
    <nav class="breadcrumb">
        <a href="/" class="breadcrumb-link">Home</a>
        {#each breadcrumbs as { label, href }, i}
            <span class="breadcrumb-separator">/</span>
            {#if i === breadcrumbs.length - 1}
                <span class="breadcrumb-text" aria-current="page">{label}</span>
            {:else}
                <a {href} class="breadcrumb-link">{label}</a>
            {/if}
        {/each}
    </nav>
    <div class="actions">
        <button class="search-button" aria-label="Search" onclick={() => (showSearchModal = true)}>
            <Search />
            <span class="keyboard-shortcut">
                <span class="command">⌘</span>
                <span class="key">K</span>
            </span>
        </button>
    </div>
</header>

{#if showSearchModal}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-backdrop" onclick={closeModal} transition:fade={{ duration: 150 }}>
        <div class="search-modal" onclick={(e) => e.stopPropagation()}>
            <div class="search-header">
                <Search />
                <input
                    bind:this={searchInput}
                    type="text"
                    placeholder="Search..."
                    class="search-input"
                />
                <button class="close-button" onclick={closeModal}>
                    <span>ESC</span>
                </button>
            </div>
            <div class="search-results">
                <div class="empty-state">
                    <p>Type to search across the application</p>
                </div>
            </div>
        </div>
    </div>
{/if}

<style>
    .top-nav {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem 2rem;
        border-bottom: 1px solid rgba(0, 0, 0, 0.1);

        .breadcrumb {
            font-size: 1.1rem;
            font-weight: 500;
            color: var(--colors-dark100);
        }

        .actions {
            display: flex;
            align-items: center;

            .search-button {
                display: flex;
                align-items: center;
                gap: 8px;
                background: none;
                border: none;
                cursor: pointer;
                padding: 0.5rem 0.75rem;
                border-radius: 6px;
                transition: background-color 0.2s;

                .keyboard-shortcut {
                    display: flex;
                    align-items: center;
                    gap: 0.125rem;
                    justify-content: center;
                    opacity: 0.7;
                    background-color: rgba(0, 0, 0, 0.05);
                    padding: 2px 6px;
                    border-radius: 0.25rem;

                    .command {
                        font-size: 0.625rem;
                    }

                    .key {
                        font-size: 0.75rem;
                    }
                }

                &:hover {
                    background-color: rgba(0, 0, 0, 0.05);
                }
            }
        }
    }

    .modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: flex-start;
        padding-top: 10vh;
        z-index: 1000;

        .search-modal {
            width: 600px;
            max-width: 90%;
            background-color: white;
            border-radius: 8px;
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
            overflow: hidden;

            .search-header {
                display: flex;
                align-items: center;
                padding: 1rem;
                border-bottom: 1px solid rgba(0, 0, 0, 0.1);

                :global(svg) {
                    margin-right: 0.75rem;
                }

                .search-input {
                    flex: 1;
                    border: none;
                    font-size: 1rem;
                    outline: none;
                    background: transparent;
                }

                .close-button {
                    background: none;
                    border: none;
                    cursor: pointer;
                    opacity: 0.6;
                    padding: 4px 8px;
                    border-radius: 4px;
                    font-size: 0.75rem;
                    background-color: rgba(0, 0, 0, 0.05);

                    &:hover {
                        opacity: 1;
                    }
                }
            }

            .search-results {
                max-height: 400px;
                overflow-y: auto;

                .empty-state {
                    padding: 2rem;
                    text-align: center;
                    opacity: 0.6;
                }
            }
        }
    }
</style>
