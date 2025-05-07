<script lang="ts">
    import Pill from '$lib/shared-components/Pill.svelte';
    import { theme } from '$lib/theme';
    import { createDialog, melt } from '@melt-ui/svelte';

    // TODO: If more modals are used in the future, should create a CreateNewStreamModal component
    // that reuses a default modal component that we'll create

    const {
        elements: { trigger, portalled, overlay, content, title, description },
        states: { open },
    } = createDialog({
        defaultOpen: false,
        forceVisible: true,
    });

    let selectedOption: 'left' | 'right' = 'left';

    function handleContinue() {
        // Add your continue logic here based on selectedOption
        console.log('Selected option:', selectedOption);
        $open = false; // Close the modal
    }
</script>

<div class="header">
    <div class="title-section">
        <h1>Tab Headline</h1>
        <p>
            We're on a mission to revolutionize video streaming solutions with cutting-edge tools
            and libraries.
        </p>
    </div>

    <div class="create-button-link">
        <button use:melt={$trigger}>
            <Pill color={theme.colors.orange500} as="div">
                <div class="create-button">
                    Create New
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="20"
                        height="20"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <circle cx="12" cy="12" r="10"></circle>
                        <path d="M12 8v8"></path>
                        <path d="M8 12h8"></path>
                    </svg>
                </div>
            </Pill>
        </button>
    </div>
</div>

{#if $open}
    <div use:melt={$portalled}>
        <div use:melt={$overlay} class="modal-overlay"></div>
        <div use:melt={$content} class="modal-content">
            <h2 use:melt={$title} class="modal-title">Choose Your Option</h2>
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
            <button class="continue-button" on:click={handleContinue}>Continue</button>
        </div>
    </div>
{/if}

<style>
    .header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 2rem;

        .title-section {
            h1 {
                font-size: 2.5rem;
                font-weight: 700;
                color: #1a1a1a;
                margin: 0 0 0.5rem 0;
            }

            p {
                font-size: 1rem;
                color: #555;
                max-width: 600px;
                margin: 0;
                line-height: 1.5;
            }
        }

        .create-button-link {
            text-decoration: none;
            flex-shrink: 0;

            button {
                padding: 0;
                border: none;
                background: none;
                cursor: pointer;
                border-radius: 9999px; /* Match pill shape */

                &:focus-visible {
                    outline: 2px solid var(--color-orange500);
                    outline-offset: 2px;
                    border-radius: 9999px;
                }
            }

            .create-button {
                display: flex;
                align-items: center;
                gap: 0.5rem;
                background-color: #ffd280;
                font-weight: 600;
            }
        }
    }

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

        .modal-title {
            text-align: center;
            margin-bottom: 2rem;
            font-size: 1.5rem;
            font-weight: 600;
        }
    }

    .options-container {
        display: flex;
        gap: 2rem;
        margin-bottom: 2rem;

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

            &.selected {
                border-color: var(--color-orange500);
                background-color: rgba(255, 210, 128, 0.1);
            }

            h3 {
                margin: 0 0 1rem 0;
                font-size: 1.25rem;
                font-weight: 600;
            }

            p {
                margin: 0;
                color: #555;
            }
        }
    }

    .continue-button {
        display: block;
        margin: 0 auto;
        padding: 0.75rem 2rem;
        background-color: var(--color-orange500);
        border-radius: 4px;
        font-weight: 600;
        cursor: pointer;
        transition: opacity 0.2s ease;
        border: 2px solid black;
        &:hover {
            opacity: 0.9;
        }
    }
</style>
