<script lang="ts">
    import VideoHeader from './video-header.svelte';
    import IconCopy from '$lib/images/icon-copy.svelte';
    let streamId = '8a28e499d6s7987fd9812937fd981...';
    let streamKey = '••••••••••••••••••••••••';
    let isConnected = true;
    let requireSignedUrls = false;

    import IconOverviewKey from '$lib/images/icon-overview-key.svelte';
    import IconCopyCheckmark from '$lib/images/icon-copy-checkmark.svelte';

    let copied = {
        streamId: false,
        streamKey: false,
    };

    function copyToClipboard(text: string, field: keyof typeof copied) {
        navigator.clipboard.writeText(text);
        copied[field] = true;

        setTimeout(() => {
            copied[field] = false;
        }, 2000);
    }
</script>

<div class="overview-tab-container">
    <VideoHeader {streamId} {streamKey} {isConnected} {requireSignedUrls} />
    <section class="keys-section">
        <div class="section-header">
            <div class="section-header-icon">
                <IconOverviewKey />
            </div>
            <h2>Important keys</h2>
        </div>
        <div class="keys-content">
            <div class="key-card">
                <div class="key-label">Stream ID</div>
                <div class="key-value">
                    <span>{streamId}</span>
                    <button
                        class="copy-button"
                        class:copied={copied.streamId}
                        onclick={() => copyToClipboard(streamId, 'streamId')}
                    >
                        {#if copied.streamId}
                            <IconCopyCheckmark />
                        {:else}
                            <IconCopy />
                        {/if}
                    </button>
                </div>
            </div>
            <div class="key-card">
                <div class="key-label">Stream Key</div>
                <div class="key-value">
                    <span class="masked-key">{streamKey}</span>
                    <button
                        class="copy-button"
                        class:copied={copied.streamKey}
                        onclick={() => copyToClipboard(streamKey, 'streamKey')}
                    >
                        {#if copied.streamKey}
                            <IconCopyCheckmark />
                        {:else}
                            <IconCopy />
                        {/if}
                    </button>
                </div>
            </div>
        </div>
    </section>

    <section class="keys-section">
        <div class="section-header">
            <div class="section-header-icon">
                <IconOverviewKey />
            </div>
            <h2>URLs</h2>
        </div>
        <div class="keys-content">
            <div class="key-card">
                <div class="key-label">Signed URLs</div>
                <div class="key-value">
                    <span>{requireSignedUrls ? 'Enabled' : 'Disabled'}</span>
                    <button
                        class="copy-button"
                        onclick={() => (requireSignedUrls = !requireSignedUrls)}
                    >
                        <input
                            type="checkbox"
                            bind:checked={requireSignedUrls}
                            style="display: none;"
                        />
                        {#if requireSignedUrls}
                            <IconCopyCheckmark />
                        {:else}
                            <IconCopy />
                        {/if}
                    </button>
                </div>
            </div>
            <div class="key-card">
                <div class="key-label">Passphrase</div>
                <div class="key-value">
                    <span class="masked-key">{streamKey}</span>
                    <button
                        class="copy-button"
                        class:copied={copied.streamKey}
                        onclick={() => copyToClipboard(streamKey, 'streamKey')}
                    >
                        {#if copied.streamKey}
                            <IconCopyCheckmark />
                        {:else}
                            <IconCopy />
                        {/if}
                    </button>
                </div>
            </div>
        </div>
    </section>
</div>

<style>
    .overview-tab-container {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    h2 {
        font-size: 1.2rem;
        margin-bottom: 1rem;
        color: #333;
    }

    .keys-section {
        margin-bottom: 2rem;
        background: var(--color-teal70);
        border-radius: 0.5rem;
        padding: 0.25rem;
        box-shadow:
            0px 1px 2px 0px #e6dedb,
            -1px 1px 0px 0px #e6dedb,
            1px 1px 0px 0px #e6dedb,
            1px -1px 0px 0px #e6dedb,
            -1px -1px 0px 0px #e6dedb;

        .section-header {
            display: flex;
            align-items: center;
            padding: 0.5rem;
            align-items: center;

            .section-header-icon {
                padding: 0.5rem;
            }

            h2 {
                margin: 0;
                color: var(--color-brown90);
                font-size: 1rem;
                font-style: normal;
                font-weight: 700;
                line-height: 1.5rem;
            }
        }

        /* Where items start */
        .keys-content {
            display: flex;
            flex-direction: column;
            gap: 0.25rem;
        }

        .key-card {
            display: flex;
            padding: 1rem;
            justify-content: space-between;
            align-items: center;
            align-self: stretch;
            background: var(--color-teal30);
            border-radius: 0.5rem;
            padding: 1rem;
            border: 1px solid var(--colors-teal-10, #fefcfb);

            .key-label {
                color: var(--colors-brown90);
                text-overflow: ellipsis;
                font-size: 1rem;
                font-style: normal;
                font-weight: 500;
                line-height: 1.5rem; /* 150% */
            }

            .key-value {
                display: flex;
                align-items: center;
                gap: 0.75rem;
                background: var(--color-teal50);
                border-radius: 0.5rem;
                padding: 0.5rem 1rem;

                span {
                    font-size: 1rem;
                    font-style: normal;
                    font-weight: 500;
                    line-height: 1.5rem; /* 150% */
                }

                .copy-button {
                    background: none;
                    border: none;
                    cursor: pointer;
                    padding: 0.25rem;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    color: #543a3c;
                    transition: color 0.2s;

                    &:hover {
                        color: #555;
                    }

                    &.copied {
                        color: #4caf50;
                    }
                }

                .masked-key {
                    font-family: monospace;
                }
            }
        }
    }

    .urls-section {
        margin-bottom: 2rem;
        background: var(--color-teal70);
        border-radius: 0.5rem;
        padding: 0.25rem;
        box-shadow:
            0px 1px 2px 0px #e6dedb,
            -1px 1px 0px 0px #e6dedb,
            1px 1px 0px 0px #e6dedb,
            1px -1px 0px 0px #e6dedb,
            -1px -1px 0px 0px #e6dedb;

        .section-header {
            display: flex;
            align-items: center;
            padding: 0.5rem;

            .section-header-icon {
                padding: 0.5rem;
                font-size: 1.2rem;
            }

            h2 {
                margin: 0;
                color: var(--color-brown90);
                font-size: 1rem;
                font-style: normal;
                font-weight: 700;
                line-height: 1.5rem;
            }
        }

        input[type='password'] {
            background: transparent;
            border: none;
            color: var(--colors-brown90);
            font-size: 1rem;
            font-family: monospace;
            width: 100%;
            outline: none;

            &::placeholder {
                color: var(--colors-brown50);
            }
        }

        .toggle {
            display: flex;
            align-items: center;
            gap: 0.5rem;

            .slider {
                position: relative;
                width: 40px;
                height: 24px;
                background: #ccc;
                border-radius: 12px;
                cursor: pointer;

                &::before {
                    content: '';
                    position: absolute;
                    width: 20px;
                    height: 20px;
                    border-radius: 50%;
                    background: white;
                    top: 2px;
                    left: 2px;
                    transition: transform 0.2s;
                }
            }

            input:checked + .slider {
                background: #4caf50;

                &::before {
                    transform: translateX(16px);
                }
            }

            input {
                display: none;
            }

            .toggle-label {
                font-size: 0.875rem;
                color: var(--colors-brown90);
            }
        }
    }
</style>
