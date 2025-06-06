<script lang="ts">
    import { Popover } from 'melt/builders';
    import IconPlus from '$lib/images/icon-plus.svelte';

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
                    <div class="option-icon webhook">📡</div>
                    <div class="option-info">
                        <h3>Webhook Stream</h3>
                        <p>Subtext about this temporary text</p>
                    </div>
                </div>
                <button class="option-button webhook-btn" on:click={createWebhookStream}>
                    View Keys ✏️
                </button>
            </div>

            <div class="stream-option">
                <div class="option-header">
                    <div class="option-icon configured">⚙️</div>
                    <div class="option-info">
                        <h3>Configured Stream</h3>
                        <p>Subtext about this temporary text</p>
                    </div>
                </div>
                <button class="option-button configured-btn" on:click={createConfiguredStream}>
                    Create Configured Stream +
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
        border-radius: 12px;
        box-shadow:
            0 10px 15px -3px rgba(0, 0, 0, 0.1),
            0 4px 6px -2px rgba(0, 0, 0, 0.05);
        padding: 1rem;
        width: 400px;
        z-index: 50;
        margin: 0;
    }

    .stream-option {
        padding: 1rem;
        border-radius: 8px;
        margin-bottom: 0.75rem;
        border: 1px solid #f3f4f6;

        &:last-child {
            margin-bottom: 0;
        }
    }

    .option-header {
        display: flex;
        align-items: flex-start;
        gap: 0.75rem;
        margin-bottom: 0.75rem;

        .option-icon {
            width: 40px;
            height: 40px;
            border-radius: 8px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 1.25rem;
            flex-shrink: 0;

            &.webhook {
                background-color: #e0e7ff;
            }

            &.configured {
                background-color: #fef3c7;
            }
        }

        .option-info {
            h3 {
                margin: 0 0 0.25rem 0;
                font-size: 1rem;
                font-weight: 600;
                color: #1f2937;
            }

            p {
                margin: 0;
                font-size: 0.875rem;
                color: #6b7280;
            }
        }
    }

    .option-button {
        padding: 0.5rem 1rem;
        border-radius: 6px;
        font-size: 0.875rem;
        font-weight: 500;
        border: none;
        cursor: pointer;

        &:focus-visible {
            outline: 2px solid var(--colors-orange100);
        }

        &.webhook-btn {
            background-color: #e0e7ff;
            color: #3730a3;

            &:hover {
                background-color: #c7d2fe;
            }
        }

        &.configured-btn {
            background-color: #fbbf24;
            color: #92400e;

            &:hover {
                background-color: #f59e0b;
            }
        }
    }
</style>
