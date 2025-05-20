<script lang="ts">
    import VideoHeader from './video-header.svelte';
    let streamId = '8a28e499d6s7987fd9812937fd981...';
    let streamKey = '••••••••••••••••••••••••';
    let isConnected = true;
    let requireSignedUrls = false;
</script>

<VideoHeader />

<section class="keys-section">
    <h2>🔑 Important keys</h2>
    <div class="key-item">
        <div class="key-label">Stream ID</div>
        <div class="key-value">
            <span>{streamId}</span>
            <button class="copy-button">📋</button>
        </div>
    </div>
    <div class="key-item">
        <div class="key-label">Stream Key</div>
        <div class="key-value">
            <span>Private</span>
            <span class="masked-key">{streamKey}</span>
            <button class="copy-button">📋</button>
        </div>
    </div>
</section>

<section class="urls-section">
    <h2>🌐 URLs</h2>
    <div class="signed-urls">
        <h3>Require Signed URLs</h3>
        <p class="subtext">Subtext about this temporary text</p>
        <label class="toggle">
            <input type="checkbox" bind:checked={requireSignedUrls} />
            <span class="slider"></span>
            <span class="toggle-label">
                {requireSignedUrls ? 'Enabled' : 'Disabled'}
            </span>
        </label>
    </div>
</section>

<style>
    .overview-container {
        padding: 1.5rem;
        background: #fff;
        border-radius: 8px;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);

        h2 {
            font-size: 1.2rem;
            margin-bottom: 1rem;
            color: #333;
        }

        .info-grid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 1rem;
            margin-bottom: 1.5rem;

            .info-item {
                background: #f5f5f5;
                padding: 0.75rem;
                border-radius: 6px;

                .label {
                    display: block;
                    font-size: 0.9rem;
                    color: #666;
                }

                .value {
                    display: block;
                    font-weight: 500;
                    margin-top: 0.25rem;
                }

                .status {
                    display: flex;
                    align-items: center;
                    gap: 0.5rem;

                    .status-dot {
                        width: 8px;
                        height: 8px;
                        border-radius: 50%;

                        &.connected {
                            background: #4caf50;
                        }
                    }
                }
            }
        }

        .webhook-info {
            background: #f0f0ff;
            padding: 1rem;
            border-radius: 6px;
            margin-top: 1rem;

            .webhook-badge {
                display: flex;
                align-items: center;
                gap: 0.5rem;
                color: #4444ff;
                margin-bottom: 0.5rem;
            }

            .webhook-notice {
                font-size: 0.9rem;
                color: #666;
            }
        }

        .encoding-grid {
            background: #f5f5f5;
            border-radius: 6px;
            padding: 1rem;

            .encoding-row {
                display: grid;
                grid-template-columns: repeat(3, 1fr);
                gap: 1rem;
                margin-bottom: 0.5rem;

                .label {
                    color: #666;
                }

                .value {
                    font-weight: 500;
                }
            }
        }

        .keys-section {
            .key-item {
                background: #f5f5f5;
                padding: 1rem;
                border-radius: 6px;
                margin-bottom: 1rem;

                .key-value {
                    display: flex;
                    align-items: center;
                    gap: 0.5rem;
                    margin-top: 0.5rem;

                    .copy-button {
                        background: none;
                        border: none;
                        cursor: pointer;
                        padding: 0.25rem;
                    }

                    .masked-key {
                        font-family: monospace;
                    }
                }
            }
        }

        .urls-section {
            .signed-urls {
                background: #f5f5f5;
                padding: 1rem;
                border-radius: 6px;

                .toggle {
                    display: flex;
                    align-items: center;
                    gap: 0.5rem;
                    margin-top: 1rem;

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
                }
            }
        }
    }
</style>
