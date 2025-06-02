<script lang="ts">
    import { Chart, type ECMouseEvent } from 'svelte-echarts';

    import { init, use } from 'echarts/core';
    import { type EChartsOption } from 'echarts';
    import { BarChart, CustomChart, LineChart, ScatterChart } from 'echarts/charts';
    import {
        DatasetComponent,
        DataZoomComponent,
        GridComponent,
        TitleComponent,
        ToolboxComponent,
        TooltipComponent,
        VisualMapComponent,
    } from 'echarts/components';

    import { CanvasRenderer } from 'echarts/renderers';
    import { renderItem } from './shape-renderers';
    import { theme } from '$lib/theme';
    import type { ChartData } from '../types';
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
        ScatterChart,
        CustomChart,
    ]);

    type Props = {
        eventDetails: ChartData | null;
    };
    const { eventDetails = null }: Props = $props();

    const startTime = new Date('2024-01-01T00:00:00').getTime();
    const categories = ['categoryC', 'categoryB', 'categoryA'];

    const data = $derived(eventDetails?.eventData || []);
    const lineData = $derived(eventDetails?.lineData || []);

    const xMinTime = startTime;
    const xMaxTime = startTime + 1000 * 60 * 2;
    // Must be used for all charts
    const xAxisOptions = {
        type: 'time',
        splitLine: {
            show: true,
            showMinLine: false,
            showMaxLine: false,
            // Only method of changing line style with echarts
            lineStyle: {
                color: {
                    type: 'linear',
                    x: 0,
                    y: 0,
                    x2: 0,
                    y2: 1,
                    colorStops: [
                        {
                            offset: 0,
                            color: 'transparent',
                        },
                        {
                            offset: 0.15,
                            color: 'transparent',
                        },
                        {
                            offset: 0.15,
                            color: theme.colors.gray40,
                        },
                        {
                            offset: 0.85,
                            color: theme.colors.gray40,
                        },
                        {
                            offset: 0.85,
                            color: 'transparent',
                        },
                        {
                            offset: 1,
                            color: 'transparent',
                        },
                    ],
                },
                width: 1,
                type: 'solid',
                cap: 'round',
            },
        },
        min: xMinTime,
        max: xMaxTime,
        axisTick: {
            show: true,
            length: 6,
            // Only method of changing tick style with echarts
            lineStyle: {
                color: {
                    type: 'linear',
                    x: 0,
                    y: 0,
                    x2: 0,
                    y2: 1,
                    colorStops: [
                        {
                            offset: 0,
                            color: theme.colors.gray60,
                        },
                        {
                            offset: 0.5,
                            color: theme.colors.gray60,
                        },
                        {
                            offset: 0.5,
                            color: 'transparent',
                        },
                        {
                            offset: 1,
                            color: 'transparent',
                        },
                    ],
                },
                width: 1,
                cap: 'round',
            },
        },
        axisLabel: {
            formatter: function (val: number) {
                const date = new Date(val);
                const formatter = new Intl.DateTimeFormat('en-US', {
                    hour: '2-digit',
                    minute: '2-digit',
                    second: '2-digit',
                    timeZone: 'America/New_York',
                    timeZoneName: 'short',
                });
                return formatter.format(date);
            },
            showMinLabel: true,
            showMaxLabel: true,
            alignMinLabel: 'left',
            alignMaxLabel: 'right',
            hideOverlap: true,
            interval: 'auto',
            color: theme.colors.gray60,
            margin: 10,
        },
    };

    const option = $derived({
        grid: [
            {
                left: '8%',
                right: '8%',
                top: '28%',
                height: '25%',
                show: true,
                backgroundColor: theme.colors.teal30,
                borderColor: theme.colors.teal30,
            },
            {
                left: '8%',
                right: '8%',
                top: '55%',
                height: '25%',
                show: true,
                backgroundColor: theme.colors.teal30,
                borderColor: theme.colors.teal30,
            },
        ],
        xAxis: [
            {
                position: 'top',
                gridIndex: 0,
                ...xAxisOptions,
            },
            {
                gridIndex: 1,
                ...xAxisOptions,
                axisLabel: {
                    ...xAxisOptions.axisLabel,
                    show: false,
                },
                axisTick: {
                    show: false,
                },
                axisLine: {
                    show: false,
                },
            },
        ],
        yAxis: [
            {
                data: categories,
                show: false,
            },
            {
                type: 'value',
                gridIndex: 1,
                show: false,
            },
        ],
        tooltip: {
            formatter: function (params: any) {
                return params.marker + params.name + ': ' + params.value[3] + ' ms';
            },
        },
        dataZoom: [
            {
                type: 'slider',
                filterMode: 'weakFilter',
                showDataShadow: false,
                top: 1,
                xAxisIndex: [0, 1],
                start: 30,
                end: 70,
                textStyle: {
                    color: '#333',
                    fontFamily: 'Arial',
                    fontSize: 12,
                    right: 50,
                },
                labelFormatter: '{value}',
                borderRadius: 4,
                fillerColor: '#EEE6E2',
                // This background color seems to ignore the borderRadius. TBD
                // backgroundColor: theme.colors.teal100,
                borderColor: theme.colors.teal100,
                handleStyle: {
                    color: theme.colors.yellow40,
                    borderColor: theme.colors.yellow40,
                    borderWidth: 1,
                    borderRadius: 4,
                },
                moveHandleStyle: {
                    color: theme.colors.brown50,
                    opacity: 0.7,
                },
            },
            {
                type: 'inside',
                filterMode: 'weakFilter',
                xAxisIndex: [0, 1],
            },
        ],
        series: [
            {
                type: 'custom',
                renderItem: renderItem,
                itemStyle: {
                    opacity: 0.8,
                },
                encode: {
                    x: [1, 2],
                    y: 0,
                },
                data,
            },
            {
                type: 'line',
                data: lineData.map((item) => [item.timestamp, item.value]),
                xAxisIndex: 1,
                yAxisIndex: 1,
                showSymbol: false,
                lineStyle: {
                    color: '#91c7dd',
                },
            },
        ],
    });

    let options = $derived({ ...option } as EChartsOption);

    const handleClick = (event: ECMouseEvent) => {
        alert(`${event.name} ${event.value}`);
    };
</script>

<div class="chart-container">
    <div class="grid-overlay top"></div>
    <div class="grid-overlay bottom"></div>
    <Chart {init} {options} onclick={handleClick} />
</div>

<style>
    .chart-container {
        position: relative;
        width: 120%;
        height: 100%;
        margin-left: -10%;
        overflow: visible;
    }

    .grid-overlay {
        position: absolute;
        left: 8%;
        right: 8%;
        height: 25%;
        border-radius: 0.25rem;
        pointer-events: none;
        z-index: 1;
        background-color: rgba(230, 222, 219, 0.6);
        mix-blend-mode: multiply;
    }

    .grid-overlay.top {
        top: 28%;
    }

    .grid-overlay.bottom {
        top: 55%;
    }
</style>
