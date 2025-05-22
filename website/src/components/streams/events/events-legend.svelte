<script lang="ts">
    import IconEventsAsset from '$lib/images/icon-events-asset.svelte';
    import IconEventsError from '$lib/images/icon-events-error.svelte';
    import IconEventsNeutral from '$lib/images/icon-events-neutral.svelte';

    const legendItems = [
        { icon: IconEventsAsset, text: 'Unexpected Error' },
        { icon: IconEventsError, text: 'Neutral Event' },
        { icon: IconEventsNeutral, text: 'Asset Created' },
    ];

    const metrics = [
        { label: 'Avg. GOP', value: '250 frames / 4.1s' },
        { label: 'Avg. Encoding video bitrate', value: '320 kbit/s' },
        { label: 'Avg. Ingress bitrate', value: '320 kbit/s' },
    ];
</script>

<div class="metrics">
    {#each metrics as metric}
        <div class="metric">
            <span class="label">{metric.label}</span>
            <span class="value">{metric.value}</span>
        </div>
    {/each}
</div>
<div class="legend">
    {#each legendItems as item}
        <div class="legend-item">
            <span class="icon">
                <svelte:component this={item.icon} />
            </span>
            <span class="text">{item.text}</span>
        </div>
    {/each}
</div>

<style>
    .metrics {
        display: flex;
        gap: 2rem;
        align-items: center;
        margin-bottom: 1rem;
        padding: 0.75rem;
        border-radius: 0.25rem;

        .metric {
            display: flex;
            flex-direction: column;
            gap: 0.25rem;
            flex: 1;

            .label {
                color: var(--color-brown70);
                font-size: 0.875rem;
                line-height: 1.25rem;
            }

            .value {
                color: var(--color-brown90);
                font-size: 1rem;
                font-weight: 500;
                line-height: 1.5rem;
            }
        }
    }
    .legend {
        display: flex;
        gap: 2rem;
        align-items: center;

        .legend-item {
            display: flex;
            align-items: center;
            gap: 0.5rem;

            .icon {
                display: flex;
                align-items: center;
                justify-content: center;
            }

            .text {
                font-size: 1rem;
                color: var(--color-brown70);
                font-size: 1rem;
                font-style: normal;
                font-weight: 500;
                line-height: 1.5rem; /* 150% */
            }
        }
    }
</style>
