<script lang="ts">
    import { Popover } from 'melt/builders';
    import IconPlus from '$lib/images/icon-plus.svelte';
    import IconWebhook from '$lib/images/icon-webhook.svelte';
    import IconStream from '$lib/images/icon-stream.svelte';
    import IconOverviewKey from '$lib/images/icon-overview-key.svelte';

    const popover = new Popover({
        computePositionOptions: {
            placement: 'bottom-end',
        },
    });

    function createWebhookStream() {
        console.log('Creating webhook stream...');
    }

    function createConfiguredStream() {
        console.log('Creating configured stream...');
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

    <div class="create-button-container">
        <button class="create-button-wrapper" {...popover.trigger}>
            <div class="create-button">
                New Stream
                <IconPlus />
            </div>
        </button>

        <div {...popover.content} class="popover-content">
            <div class="stream-option">
                <div class="option-header">
                    <div class="option-title">
                        <IconWebhook />
                        <h3>Webhook Stream</h3>
                    </div>
                    <div class="option-info">
                        <p>
                            Create a stream that will be triggered by a webhook. See our
                            documentation for more information.
                        </p>
                    </div>
                </div>
                <button class="option-button webhook-btn" on:click={createWebhookStream}>
                    View Keys <IconOverviewKey />
                </button>
            </div>

            <div class="stream-option">
                <div class="option-header">
                    <div class="option-title">
                        <IconStream />
                        <h3>Configured Stream</h3>
                    </div>
                    <div class="option-info">
                        <p>Create a new configurable stream</p>
                    </div>
                </div>
                <button class="option-button configured-btn" on:click={createConfiguredStream}>
                    Create Configured Stream <IconPlus />
                </button>
            </div>
        </div>
    </div>
</div>

<style>
    .header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 2rem;
    }

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

    .create-button-container {
        position: relative;
        flex-shrink: 0;
    }

    .create-button-wrapper {
        padding: 0;
        border: none;
        background: none;
        cursor: pointer;
        border-radius: 9999px;

        &:focus-visible {
            outline: 2px solid var(--colors-orange500);
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
        padding: 0.75rem 1.5rem;
        border-radius: 9999px;
    }

    .popover-content {
        background: white;
        border: 1px solid #e5e7eb;
        border-radius: 0.5rem;
        box-shadow:
            0 10px 15px -3px rgba(0, 0, 0, 0.1),
            0 4px 6px -2px rgba(0, 0, 0, 0.05);
        padding: 0.5rem;
        width: 44rem;
        z-index: 50;
        margin: 0;
        display: flex;
        gap: 0.5rem;
    }

    .stream-option {
        padding: 0.5rem;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        gap: 0.5rem;
        flex: 1;
    }

    .option-header {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 0.25rem;
        padding: 0.375rem;

        .option-title {
            display: flex;
            align-items: center;
            flex-direction: row;
            gap: 0.5rem;

            h3 {
                font-size: 1rem;
                font-weight: 700;
                line-height: 1.5rem;
            }
        }

        .option-info {
            display: flex;
            flex-direction: column;
            color: var(--colors-brown90);
            font-size: 1rem;
            font-weight: 500;
            line-height: normal;
        }
    }

    .option-button {
        padding: 0.5rem 1rem;
        border-radius: 6px;
        font-size: 0.875rem;
        font-weight: 500;
        border: none;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;

        &:focus-visible {
            outline: 2px solid var(--colors-orange100);
        }

        &.webhook-btn {
            color: var(--colors-yellow90);
            font-size: 1rem;
            font-weight: 700;
            line-height: 1.5rem;
            background-color: var(--colors-purple10);
            border-radius: 0.5rem;
            border: 1px solid var(--colors-purple30);
            background: var(--colors-purple30);

            &:hover {
                background-color: #c7d2fe;
            }
        }

        &.configured-btn {
            color: var(--colors-yellow90);
            font-size: 1rem;
            font-weight: 700;
            line-height: 1.5rem;
            background-color: var(--colors-yellow10);
            border-radius: 0.5rem;
            border: 1px solid var(--colors-yellow30);
            background: var(--colors-yellow30);

            &:hover {
                background-color: #f59e0b;
            }
        }
    }

    [data-melt-popover-content] {
        position: absolute;
        pointer-events: none;
        opacity: 0;
        transform: scale(0.9);
        transition: 0.3s;
        transition-property: opacity, transform;
    }

    [data-melt-popover-content][data-open] {
        pointer-events: auto;
        opacity: 1;
        transform: scale(1);
    }
</style>
