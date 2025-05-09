<script lang="ts">
    import IconDots from '$lib/images/IconDots.svelte';
    let credentials = {
        accountId: 'your-account-id-here',
        streamToken: 'your-stream-token-here',
    };

    let copied = {
        accountId: false,
        streamToken: false,
    };

    function copyToClipboard(text: string, field: keyof typeof copied) {
        navigator.clipboard.writeText(text);
        copied[field] = true;

        setTimeout(() => {
            copied[field] = false;
        }, 2000);
    }

    // Function to mask sensitive information
    function maskText(text: string) {
        return '●'.repeat(9);
    }
</script>

<div class="api-information">
    <div class="header-row">
        <div class="user-icon">
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="currentColor"
                width="24"
                height="24"
            >
                <path
                    d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z"
                />
            </svg>
        </div>
        <h2>Account Details</h2>
        <IconDots />
    </div>

    <div class="credential-section-container">
        <div class="credential-section">
            <div class="label">Account ID</div>
            <div class="input-row">
                <div class="masked-input">{maskText(credentials.accountId)}</div>
                <button
                    class="copy-button"
                    class:copied={copied.accountId}
                    onclick={() => copyToClipboard(credentials.accountId, 'accountId')}
                >
                    {#if copied.accountId}
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="currentColor"
                            width="20"
                            height="20"
                        >
                            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z" />
                        </svg>
                    {:else}
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="currentColor"
                            width="20"
                            height="20"
                        >
                            <path
                                d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"
                            />
                        </svg>
                    {/if}
                </button>
            </div>
        </div>

        <div class="credential-section">
            <div class="label">Stream Token</div>
            <div class="input-row">
                <div class="masked-input">{maskText(credentials.streamToken)}</div>
                <button
                    class="copy-button"
                    class:copied={copied.streamToken}
                    onclick={() => copyToClipboard(credentials.streamToken, 'streamToken')}
                >
                    {#if copied.streamToken}
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="currentColor"
                            width="20"
                            height="20"
                        >
                            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z" />
                        </svg>
                    {:else}
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="currentColor"
                            width="20"
                            height="20"
                        >
                            <path
                                d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"
                            />
                        </svg>
                    {/if}
                </button>
            </div>
        </div>
    </div>
</div>

<style>
    .api-information {
        background-color: #a5a5a540;
        border-radius: 10px;
        padding: 0.125rem;
        width: 100%;

        .header-row {
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 0.5rem;

            .user-icon {
                color: #333;
                margin-right: 10px;
                display: flex;
            }

            h2 {
                font-size: 1rem;
                font-weight: 500;
                color: #201617;
                flex-grow: 1;
                margin: 0;
                line-height: 1.5rem;
            }
        }

        .credential-section-container {
            background-color: var(--color-tan700);
            border-radius: 0.5rem;
            padding: 0.5rem;
            display: flex;
            flex-direction: column;
            gap: 0.5rem;

            .label {
                font-size: 16px;
                color: #555;
                margin-bottom: 8px;
            }

            .input-row {
                display: flex;
                align-items: center;
                background-color: white;
                border-radius: 8px;
                border: 1px solid #ddd;
                overflow: hidden;

                .masked-input {
                    flex-grow: 1;
                    padding: 12px;
                    font-size: 1rem;
                    line-height: 1.5rem;
                    letter-spacing: 1px;
                    font-weight: 400;
                    color: #444;
                }

                .copy-button {
                    background: none;
                    border: none;
                    padding: 12px;
                    cursor: pointer;
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
            }
        }
    }
</style>
