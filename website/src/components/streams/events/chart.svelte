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
    import { circleData, getSampleLineData, rectangleData, diamondData } from './sample-data';
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

    const startTime = new Date('2024-01-01T00:00:00').getTime();
    const categories = ['categoryC', 'categoryB', 'categoryA'];

    // Generate mock data. This should come from the backend, not sure how formatted it will be though
    // If we're 100% re-using api data.
    const data = [...rectangleData(), ...diamondData(), ...circleData()];

    const lineData = getSampleLineData();

    const xMinTime = startTime;
    const xMaxTime = startTime + 1000 * 60 * 2;
    // Must be used for all charts
    const xAxisOptions = {
        type: 'time',
        splitLine: {
            show: true,
        },
        min: xMinTime,
        max: xMaxTime,
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
        },
    };

    const option = {
        grid: [
            {
                left: '1%',
                right: '1%',
                top: '28%',
                height: '25%',
                show: true,
            },
            {
                left: '1%',
                right: '1%',
                top: '55%',
                height: '25%',
                show: true,
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
                top: 5,
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
                // fillerColor: '#EEE7E2', // color of selected area
                borderRadius: 4,
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
    };

    let options = $derived({ ...option } as EChartsOption);

    const handleClick = (event: ECMouseEvent) => {
        alert(`${event.name} ${event.value}`);
    };
</script>

<Chart {init} {options} onclick={handleClick} />

<style>
</style>
