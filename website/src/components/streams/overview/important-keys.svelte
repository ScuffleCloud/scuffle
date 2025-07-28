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

<style>
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
</style>
