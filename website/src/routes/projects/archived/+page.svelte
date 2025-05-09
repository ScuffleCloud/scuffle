<script lang="ts">
    import { onMount } from 'svelte';
    import { Chart, type ECMouseEvent } from 'svelte-echarts';

    import { init, use } from 'echarts/core';
    import type { EChartsOption } from 'echarts';
    import { BarChart } from 'echarts/charts';
    import {
        DatasetComponent,
        GridComponent,
        TitleComponent,
        TooltipComponent,
        TransformComponent,
    } from 'echarts/components';
    import { CanvasRenderer } from 'echarts/renderers';

    function formatBytes(bytes: number, decimals = 2) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const dm = decimals < 0 ? 0 : decimals;
        const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
    }

    const getTransferSize = async () =>
        performance
            .getEntriesByType('resource')
            // @ts-expect-error
            .reduce((acc, resource) => acc + resource.transferSize, 0);

    onMount(() => {});

    use([
        BarChart,
        DatasetComponent,
        GridComponent,
        TooltipComponent,
        TransformComponent,
        CanvasRenderer,
        TitleComponent,
    ]);

    let interval = 1000;
    let numRecords = 10;

    let data: { timestamp: Date; value: number }[] = $state(
        Array.from({ length: numRecords }, (_, i) => ({
            timestamp: new Date(Date.now() - (numRecords - i) * interval),
            value: Math.random() * 100,
        })),
    );

    let options = $derived({
        title: {
            text: 'ECharts example',
        },
        xAxis: {
            type: 'category',
            axisLabel: {
                formatter: (value) =>
                    new Date(value).toLocaleTimeString('en-US', {
                        hour12: false,
                        hour: '2-digit',
                        minute: '2-digit',
                        second: '2-digit',
                    }),
            },
            data: data.map(({ timestamp }) => timestamp),
        },
        yAxis: {
            type: 'value',
        },
        series: [
            {
                type: 'bar',
                data: data.map(({ value }) => value),
            },
        ],
    } as EChartsOption);

    const updateData = () => {
        data.shift();
        data.push({
            timestamp: new Date(),
            value: Math.random() * 100,
        });
        data = [...data];
    };

    const handleClick = (event: ECMouseEvent) => {
        alert(`${event.name} ${event.value}`);
    };

    onMount(() => {
        const id = setInterval(updateData, interval);
        return () => clearInterval(id);
    });
</script>

<svelte:head>
    <title>Examples - svelte-echarts</title>
</svelte:head>
<div class="app">
    {#await getTransferSize()}
        <span>Loading...</span>
    {:then bytes}
        <span>Transfer Size: {formatBytes(bytes)}</span>
        <span>Refresh without cache to see real size (CTRL+F5)</span>
    {:catch error}
        <span>Error: {error.message}</span>
    {/await}
</div>

<Chart {init} {options} onclick={handleClick} />

<style>
    .app {
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
    }
</style>
