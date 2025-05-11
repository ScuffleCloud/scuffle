<script lang="ts">
    import { onMount } from 'svelte';
    import { Chart, type ECMouseEvent } from 'svelte-echarts';

    import { init, use } from 'echarts/core';
    import type {
        CustomSeriesRenderItemAPI,
        CustomSeriesRenderItemParams,
        CustomSeriesRenderItemReturn,
        EChartsOption,
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

    let data: { timestamp: Date; value: number }[] = $state(
        Array.from({ length: numRecords }, (_, i) => ({
            timestamp: new Date(Date.now() - (numRecords - i) * interval),
            value: Math.random() * 100,
        })),
    );

    const option = {
        title: {
            text: 'Rent Periods',
        },
        tooltip: {
            trigger: 'axis',
            axisPointer: {
                type: 'shadow',
            },
            formatter: function (params: any) {
                const dataIndex = params[0].dataIndex;
                const data = params[0].data;
                return `Period ${dataIndex + 1}: ${data[0]} to ${data[1]}`;
            },
        },
        grid: {
            left: '3%',
            bottom: '3%',
            containLabel: true,
        },
        xAxis: {
            type: 'value',
            max: 11,
        },
        yAxis: {
            type: 'category',
            data: ['Rent'],
        },
        dataZoom: [
            {
                type: 'inside',
                // realtime: true,
                xAxisIndex: [0, 1],
                start: 10,
                end: 90,
                filterMode: 'empty',
            },
            {
                show: true,
                xAxisIndex: [0, 1],
                type: 'slider',
                // realtime: true,
                top: 10,
                start: 10,
                end: 90,
                filterMode: 'empty',
            },
        ],
        series: [
            {
                type: 'custom',
                // Maybe change these latter but otherwise renderItem isn't called on elements
                // whose x values have exited the view
                renderItem: (
                    _params: CustomSeriesRenderItemParams,
                    api: CustomSeriesRenderItemAPI,
                ): CustomSeriesRenderItemReturn => {
                    console.log('renderItem called');
                    console.log('test logging2');
                    console.log(api.value(0), api.value(1));
                    if (!api.size || !api.coord) return null;

                    const start = Number(api.value(0));
                    const end = Number(api.value(1));

                    const height = 0.4;

                    // Get the coordinate system
                    const categoryIndex = 0; // Since we only have one category 'Rent'

                    const points = [
                        api.coord([start, categoryIndex]),
                        api.coord([end, categoryIndex]),
                    ];
                    // This number is arbirtary. Might need to adjust later
                    const viewWidth = api.getWidth() - 90;
                    const size = api.size([0, 1]);
                    const categoryHeight = Array.isArray(size) ? size[1] : size;

                    // Width can't be greater than viewWidth of chart
                    const width = Math.min(points[1][0] - points[0][0], viewWidth - points[0][0]);

                    const rectShape = {
                        x: points[0][0],
                        y: points[0][1] - (height * categoryHeight) / 2,
                        width: width,
                        height: height * categoryHeight,
                        r: 3,
                    };

                    return {
                        type: 'group',
                        children: [
                            {
                                type: 'rect',
                                shape: rectShape,
                                style: {
                                    fill: '#91c7dd',
                                    stroke: '#2b7fa4', // Border color
                                    lineWidth: 1, // Border width
                                },
                            },
                            {
                                type: 'text',
                                style: {
                                    // Get this from params
                                    text: `${start} - ${end}`,
                                    fill: '#003043',
                                    x: points[0][0] + width / 2,
                                    y: points[0][1],
                                    align: 'center',
                                    verticalAlign: 'middle',
                                    width: width - 10,
                                    overflow: 'truncate',
                                    ellipsis: '.',
                                },
                            },
                        ],
                    };
                },
                data: [
                    [1, 3], // First bar: starts at x=1, ends at x=3
                    [4, 7], // Second bar: starts at x=4, ends at x=7
                    [8, 10], // Third bar: starts at x=8, ends at x=10
                ],
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
