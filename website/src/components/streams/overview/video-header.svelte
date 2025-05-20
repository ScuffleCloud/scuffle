<script lang="ts">
    import IconOverviewCheckSmall from '$lib/images/icon-overview-check-small.svelte';
    import IconOverviewEncoding from '$lib/images/icon-overview-encoding.svelte';
    import IconOverviewFilm from '$lib/images/icon-overview-film.svelte';
    import IconWebhook from '$lib/images/IconWebhook.svelte';

    type Props = {
        streamId: string;
        streamKey: string;
        isConnected: boolean;
        requireSignedUrls: boolean;
    };

    const { streamId, streamKey, isConnected = true, requireSignedUrls = false }: Props = $props();
</script>

<div class="container-wrapper">
    <div class="overview-container">
        <div class="video-section">
            <video
                src="placeholder"
                controls
                poster="/placeholder-thumbnail.jpg"
                class="stream-video"
            >
                <track kind="captions" />
            </video>
        </div>
        <div class="info-section">
            <div class="info-header">
                <IconOverviewFilm />
                <h2>Essential information</h2>
            </div>

            <div class="info-grid">
                <div class="info-item">
                    <span class="label">Created Date</span>
                    <span class="value">5/5/2025</span>
                </div>
                <div class="info-item">
                    <span class="label">Ending Date</span>
                    <span class="value">Ongoing</span>
                </div>
                <div class="info-item">
                    <span class="label">Status</span>
                    <span class="value">
                        {#if isConnected}
                            <IconOverviewCheckSmall />
                            Connected
                        {/if}
                    </span>
                </div>
                <div class="info-item">
                    <span class="label">VOD / Assets</span>
                    <span class="value">Enabled</span>
                </div>
            </div>

            <div class="webhook-container">
                <div class="webhook-border"></div>
                <div class="webhook-info">
                    <span class="webhook-header">
                        <IconWebhook />
                        Created using Webhook
                    </span>
                    <p class="webhook-notice">
                        Data displayed cannot be modified unless a new stream is created.
                    </p>
                </div>
            </div>
        </div>
    </div>
    <section class="encoding-section">
        <div class="encoding-header">
            <div class="encoding-header-icon">
                <IconOverviewEncoding />
            </div>
            <h2>Encoding</h2>
        </div>
        <div class="encoding-grid">
            <div class="encoding-item">
                <span class="label">Protocol</span>
                <span class="value">Protocol Name</span>
            </div>
            <div class="encoding-item">
                <span class="label">Ingress bitrate</span>
                <span class="value">320 kbit/s</span>
            </div>
            <div class="encoding-item">
                <span class="label">Encoding video bitrate</span>
                <span class="value">320 kbit/s</span>
            </div>
            <div class="encoding-item">
                <span class="label">Time elapsed</span>
                <span class="value">1:20:23 sec</span>
            </div>
            <div class="encoding-item">
                <span class="label">Encoding audio bitrate</span>
                <span class="value">320 kbit/s</span>
            </div>
            <div class="encoding-item">
                <span class="label">GOP</span>
                <span class="value">250 frames / 4.1s</span>
            </div>
        </div>
    </section>
</div>

<style>
    .container-wrapper {
        display: flex;
        flex-direction: column;
    }

    .overview-container {
        padding: 0.0625rem;
        background: #fff;
        border-radius: 8px;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        display: flex;
        gap: 0.625rem;

        .video-section {
            flex: 1;
            padding-right: 1.5rem;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 400px;

            .stream-video {
                width: 100%;
                aspect-ratio: 16 / 9;
                background: #f0f0f0;
                border-radius: 6px;
                object-fit: cover;
            }
        }

        .info-section {
            flex: 1;
            padding: 0.5rem;

            .info-header {
                display: flex;
                align-items: center;
                gap: 0.5rem;
                padding: 0.5rem;

                h2 {
                    color: #201617;
                    font-size: 1rem;
                    font-style: normal;
                    font-weight: 700;
                    line-height: 1.5rem;
                    margin: 0;
                }
            }
        }

        .info-grid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            padding: 0.25rem;
            gap: 0.25rem;
            margin-bottom: 1.5rem;

            .info-item {
                background: var(--color-teal60);
                padding: 0.5rem 1rem;
                border-radius: 0.625rem;
                display: flex;
                flex-direction: column;
                gap: 0.25rem;

                .label {
                    display: block;
                    font-size: 0.9rem;
                    color: var(--color-brown70);
                }

                .value {
                    color: var(--color-brown90);
                    font-size: 1rem;
                    font-style: normal;
                    font-weight: 700;
                    line-height: 1.5rem;
                    display: flex;
                    align-items: center;
                    gap: 0.375rem;
                }
            }
        }

        .webhook-container {
            display: flex;
            flex-direction: row;

            .webhook-border {
                border-left: 3px solid var(--color-purple50);
                width: 0.1875rem;
                align-self: stretch;
                border-radius: 0.25rem 0 0 0.25rem;
            }

            .webhook-info {
                background: #f9f5ff;
                padding: 0.5rem 1rem;
                position: relative;
                display: flex;
                flex-direction: column;
                gap: 0.75rem;

                .webhook-header {
                    display: flex;
                    align-items: center;
                    gap: 0.75rem;
                    color: #6b46c1;
                    font-weight: 500;
                    margin-bottom: 0.5rem;
                    color: var(--color-brown90);
                    font-size: 16px;
                    font-style: normal;
                    font-weight: 700;
                    line-height: 24px; /* 150% */
                }

                .webhook-notice {
                    color: var(--color-brown90);
                    font-size: 1rem;
                    font-style: normal;
                    font-weight: 500;
                    line-height: 1.5rem;
                }
            }
        }
    }
    .encoding-section {
        padding: 0.75rem 0.5rem 0.5rem 0.5rem;
        border-radius: 0rem 0rem 0.5rem 0.5rem;
        background: var(--color-teal70);

        .encoding-header {
            display: flex;
            align-items: center;
            gap: 0.5rem;

            h2 {
                color: #201617;
                font-size: 1rem;
                font-style: normal;
                font-weight: 700;
                line-height: 1.5rem;
            }

            .encoding-header-icon {
                padding: 0.5rem;
            }
        }

        .encoding-grid {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            grid-template-rows: repeat(2, 1fr);
            gap: 1rem;
            padding: 0.75rem;

            .encoding-item {
                display: flex;
                flex-direction: column;

                .label {
                    color: var(--color-brown70);
                    font-size: 1rem;
                    font-style: normal;
                    font-weight: 500;
                    line-height: 1.5rem;
                }

                .value {
                    color: var(--color-brown90);
                    font-size: 1rem;
                    font-style: normal;
                    font-weight: 700;
                    line-height: 1.5rem;
                }
            }
        }
    }
</style>
