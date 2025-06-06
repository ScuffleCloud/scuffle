<script lang="ts">
    import { createMutation } from '@tanstack/svelte-query';
    import { goto } from '$app/navigation';
    import { melt } from '@melt-ui/svelte';

    export let open: boolean;
    export let portalled: any;
    export let overlay: any;
    export let content: any;
    export let title: any;

    let selectedOption: 'left' | 'right' = 'left';

    // Might need this later but don't need dialogs right now
    // const {
    //     elements: { trigger, portalled, overlay, content, close, title },
    //     states: { open },
    // } = createDialog({
    //     defaultOpen: false,
    //     forceVisible: true,
    // });

    const mutation = createMutation({
        mutationFn: async () => {
            const response = await fetch('/api/v1/video-streams/new', {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
            });

            const data = await response.json();
            return data;
        },
        onSuccess: (data) => {
            console.log('Successfully created stream:', data);
            // onOpenChange(false);
            goto(`/streams/${data.newId}`);
        },
        onError: (error) => {
            console.error('Error creating stream:', error);
        },
    });
</script>

{#if open}
    <div use:melt={portalled}>
        <div use:melt={overlay} class="modal-overlay"></div>
        <div use:melt={content} class="modal-content">
            <h2 use:melt={title} class="modal-title">Choose Your Option</h2>
            <div class="options-container">
                <button
                    class="option-box {selectedOption === 'left' ? 'selected' : ''}"
                    on:click={() => (selectedOption = 'left')}
                >
                    <h3>Left Option</h3>
                    <p>Description for left option</p>
                </button>
                <button
                    class="option-box {selectedOption === 'right' ? 'selected' : ''}"
                    on:click={() => (selectedOption = 'right')}
                >
                    <h3>Right Option</h3>
                    <p>Description for right option</p>
                </button>
            </div>
            <button
                class="continue-button"
                on:click={() => $mutation.mutate()}
                disabled={$mutation.isPending}
            >
                {#if $mutation.isPending}
                    Creating...
                {:else}
                    Continue
                {/if}
            </button>
        </div>
    </div>
{/if}

<style>
    /* Some of this modal stuff should be made generic somewhere probably */
    .modal-overlay {
        background-color: rgba(0, 0, 0, 0.5);
        position: fixed;
        inset: 0;
        z-index: 50;
    }

    .modal-content {
        position: fixed;
        top: 30%;
        left: 50%;
        transform: translate(-50%, -50%);
        background: white;
        padding: 2rem;
        border-radius: 8px;
        z-index: 51;
        width: 90%;
        max-width: 800px;
    }

    .modal-title {
        text-align: center;
        margin-bottom: 2rem;
        font-size: 1.5rem;
        font-weight: 600;
    }

    .options-container {
        display: flex;
        gap: 2rem;
        margin-bottom: 2rem;
    }

    .option-box {
        flex: 1;
        padding: 2rem;
        border: 2px solid #e5e5e5;
        border-radius: 8px;
        text-align: center;
        cursor: pointer;
        transition: all 0.2s ease;
        background: none;
        width: 100%;
    }

    .option-box.selected {
        border-color: var(--colors-orange500);
        background-color: rgba(255, 210, 128, 0.1);
    }

    .option-box h3 {
        margin: 0 0 1rem 0;
        font-size: 1.25rem;
        font-weight: 600;
    }

    .option-box p {
        margin: 0;
        color: #555;
    }

    .continue-button {
        display: block;
        margin: 0 auto;
        padding: 0.75rem 2rem;
        background-color: var(--colors-orange500);
        border-radius: 4px;
        font-weight: 600;
        cursor: pointer;
        transition: opacity 0.2s ease;
        border: 2px solid black;
    }

    .continue-button:hover {
        opacity: 0.9;
    }

    .continue-button:disabled {
        opacity: 0.7;
        cursor: not-allowed;
    }
</style>
