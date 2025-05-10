<script lang="ts">
    import { onMount } from 'svelte';
    import { Chart, type ECMouseEvent } from 'svelte-echarts';

    import { init, use } from 'echarts/core';
    import type { EChartsOption } from 'echarts';
    import { BarChart, LineChart } from 'echarts/charts';
    import {
        DatasetComponent,
        DataZoomComponent,
        GridComponent,
        TitleComponent,
        ToolboxComponent,
        TooltipComponent,
        TransformComponent,
        VisualMapComponent,
    } from 'echarts/components';

    import { CanvasRenderer } from 'echarts/renderers';

    use([
        DatasetComponent,
        TitleComponent,
        ToolboxComponent,
        TooltipComponent,
        GridComponent,
        VisualMapComponent,
        DataZoomComponent,
        BarChart,
        CanvasRenderer,
        LineChart,
    ]);

    let interval = 1000;
    let numRecords = 30;

    let data: { timestamp: Date; value: number }[] = $state(
        Array.from({ length: numRecords }, (_, i) => ({
            timestamp: new Date(Date.now() - (numRecords - i) * interval),
            value: Math.random() * 100,
        })),
    );

    let options = $derived({
        dataset: {
            source: data,
        },
        grid: [
            {
                left: 60,
                right: 50,
                height: '35%',
            },
            {
                left: 60,
                right: 50,
                top: '55%',
                height: '35%',
            },
        ],
        xAxis: [
            {
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
            },
            {
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
                gridIndex: 1,
            },
        ],
        yAxis: [
            {
                type: 'value',
                gridIndex: 0,
            },
            {
                type: 'value',
                gridIndex: 1,
            },
        ],

        dataZoom: [
            {
                type: 'inside',
                // realtime: true,
                xAxisIndex: [0, 1],
                start: 10,
                end: 90,
            },
            {
                show: true,
                xAxisIndex: [0, 1],
                type: 'slider',
                // realtime: true,
                top: 10,
                start: 10,
                end: 90,
            },
        ],
        series: [
            {
                type: 'bar',
                encode: {
                    x: 'timestamp',
                    y: 'value',
                },
            },
            {
                type: 'line',
                encode: {
                    x: 'timestamp',
                    y: 'value',
                },
                xAxisIndex: 1,
                yAxisIndex: 1,
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
</script>

<svelte:head>
    <title>svelte-echarts Example</title>
</svelte:head>

<Chart {init} {options} onclick={handleClick} />

<style>
</style>
