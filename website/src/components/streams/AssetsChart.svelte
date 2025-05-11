<script lang="ts">
    import { onMount } from 'svelte';
    import { Chart, type ECMouseEvent } from 'svelte-echarts';

    import { init, use } from 'echarts/core';
    import {
        graphic,
        type CustomSeriesRenderItemAPI,
        type CustomSeriesRenderItemParams,
        type CustomSeriesRenderItemReturn,
        type EChartsOption,
    } from 'echarts';
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

    let interval = 1000;
    let numRecords = 30;

    // Let's attempt to use rich label
    let data: any[] = [];
    let dataCount = 10;
    let startTime = +new Date();
    let categories = ['categoryA', 'categoryB'];
    let types = [
        { name: 'JS Heap', color: '#91c7dd' },
        { name: 'Documents', color: '#bd6d6c' },
    ];

    // Generate mock data
    categories.forEach(function (category, index) {
        let baseTime = startTime;
        for (let i = 0; i < dataCount; i++) {
            let typeItem = types[Math.round(Math.random() * (types.length - 1))];
            let duration = Math.round(Math.random() * 10000);
            data.push({
                name: typeItem.name,
                value: [index, baseTime, (baseTime += duration), duration],
                itemStyle: {
                    normal: {
                        color: typeItem.color,
                    },
                },
            });
            baseTime += Math.round(Math.random() * 2000);
        }
    });

    const renderItem = (params: CustomSeriesRenderItemParams, api: CustomSeriesRenderItemAPI) => {
        if (!api.size || !api.coord) return null;
        const categoryIndex = api.value(0);
        const start = api.coord([api.value(1), categoryIndex]);
        const end = api.coord([api.value(2), categoryIndex]);

        const size = (api.size([0, 1]) as number[])[1];
        const height = size * 0.3;

        const rectShape = graphic.clipRectByRect(
            {
                x: start[0],
                y: start[1] - height / 2,
                width: end[0] - start[0],
                height: height,
            },
            {
                x: (params.coordSys as any).x,
                y: (params.coordSys as any).y,
                width: (params.coordSys as any).width,
                height: (params.coordSys as any).height,
            },
        );

        if (!rectShape) return null;

        return {
            type: 'rect',
            transition: ['shape'],
            shape: rectShape,
            style: {
                ...api.style(),
                stroke: '#2b7fa4',
                lineWidth: 1,
            },
        };
    };

    const option = {
        tooltip: {
            formatter: function (params: any) {
                return params.marker + params.name + ': ' + params.value[3] + ' ms';
            },
        },
        grid: {
            left: '3%',
            bottom: '3%',
            containLabel: true,
        },
        xAxis: {
            min: startTime,
            position: 'top',
            axisLabel: {
                formatter: function (val: number) {
                    return Math.max(0, val - startTime) + ' ms';
                },
            },
        },
        yAxis: {
            data: categories,
        },
        dataZoom: [
            {
                type: 'slider',
                filterMode: 'weakFilter',
                showDataShadow: false,
                top: 5,
            },
            {
                type: 'inside',
                filterMode: 'weakFilter',
            },
        ],
        series: [
            {
                type: 'custom',
                renderItem,
                itemStyle: {
                    opacity: 0.8,
                },
                encode: {
                    x: [1, 2],
                    y: 0,
                },
                data,
            },
        ],
    };

    let options = $derived({ ...option } as EChartsOption);

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
