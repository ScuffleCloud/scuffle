<script lang="ts">
    import type { Streamed } from '$lib/types';
    import type { VideoStream } from './types';

    type Props = {
        streamedData: Streamed<VideoStream>;
    };

    const { streamedData }: Props = $props();

    function formatCreationTime(timestamp: string) {
        const created = new Date(timestamp);
        const now = new Date();
        const diffMinutes = Math.floor((now.getTime() - created.getTime()) / (1000 * 60));
        return `${diffMinutes} minutes ago`;
    }
</script>

<!-- Vibe coded. Just placeholder -->
<div class="stream-page">
    {#await streamedData}
        <div class="loading">Loading...</div>
    {:then stream}
        <div class="stream-container">
            <div class="stream-header">
                <span class="live-indicator">• Live</span>
                <h1 class="stream-title">{stream.name}</h1>
                <div class="more-options">•••</div>
            </div>

            <div class="stream-video-container">
                <iframe
                    src={`https://www.youtube.com/embed/9bZkp7q19f0`}
                    title="YouTube video player"
                    frameborder="0"
                    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                    allowfullscreen
                    class="stream-video"
                ></iframe>
            </div>

            <div class="stream-info">
                <div class="stream-metadata">
                    <div class="metadata-row">
                        <div class="metadata-item">
                            <div class="metadata-label">Created:</div>
                            <div class="metadata-value">
                                {formatCreationTime(stream.created)}
                            </div>
                        </div>
                    </div>

                    <div class="metadata-row">
                        <div class="metadata-item">
                            <div class="metadata-label">Duration</div>
                            <div class="metadata-value">{'1:23:21'}</div>
                        </div>

                        <div class="metadata-item">
                            <div class="metadata-label">Health</div>
                            <div class="metadata-value">Good</div>
                        </div>

                        <div class="metadata-item">
                            <div class="metadata-label">Viewers</div>
                            <div class="metadata-value">{'Undefined'}</div>
                        </div>
                    </div>
                </div>

                <div class="stream-technical-info">
                    <div class="technical-row">
                        <div class="technical-item">
                            <div class="technical-label">BITRATE</div>
                            <div class="technical-graph">
                                <div class="graph-value">{'120MBS'}</div>
                                <div class="graph-visual bitrate-graph"></div>
                            </div>
                        </div>

                        <div class="technical-item">
                            <div class="technical-label">FRAMERATE</div>
                            <div class="technical-graph">
                                <div class="graph-value">{'60fps'}</div>
                                <div class="graph-visual framerate-graph"></div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="stream-details">
                    <div class="details-row">
                        <div class="details-item">
                            <div class="details-label">Stream ID:</div>
                            <div class="details-value">{stream.id}</div>
                        </div>
                    </div>

                    <div class="details-row">
                        <div class="details-item">
                            <div class="details-label">URL Stream:</div>
                            <div class="details-value">
                                {'https://www.youtube.com/watch?v=dQw4w9WgXcQ'}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    {:catch error}
        <div class="error">Error: {error.message}</div>
    {/await}
</div>

<style>
    .stream-page {
        width: 100%;
        max-width: 1200px;
        margin: 0 auto;
        padding: 20px;

        .loading,
        .error {
            display: flex;
            justify-content: center;
            align-items: center;
            height: 400px;
            font-size: 18px;
            color: #666;
        }

        .error {
            color: #e53935;
        }

        .stream-container {
            width: 100%;

            .stream-header {
                display: flex;
                align-items: center;
                margin-bottom: 20px;

                .live-indicator {
                    color: red;
                    font-weight: bold;
                    margin-right: 10px;
                }

                .stream-title {
                    margin: 0;
                    flex-grow: 1;
                }

                .more-options {
                    cursor: pointer;
                }
            }

            .stream-video-container {
                width: 100%;
                background-color: #f0f0f0;
                margin-bottom: 20px;
                aspect-ratio: 16/9;

                .stream-video {
                    width: 100%;
                    height: 100%;
                }
            }

            .stream-info {
                display: flex;
                flex-direction: column;
                gap: 20px;

                .metadata-row,
                .technical-row,
                .details-row {
                    display: flex;
                    justify-content: space-between;
                    margin-bottom: 10px;

                    .metadata-item,
                    .technical-item,
                    .details-item {
                        margin-right: 20px;

                        .metadata-label,
                        .technical-label,
                        .details-label {
                            font-weight: bold;
                            margin-bottom: 5px;
                        }
                    }
                }

                .stream-technical-info {
                    .technical-item {
                        .technical-graph {
                            .graph-visual {
                                height: 20px;
                                border-radius: 4px;
                                margin-top: 5px;
                            }

                            .bitrate-graph {
                                background: linear-gradient(90deg, #4caf50, #8bc34a);
                            }

                            .framerate-graph {
                                background: linear-gradient(90deg, #ff5252, #ff8a80);
                            }
                        }
                    }
                }
            }
        }
    }
</style>
