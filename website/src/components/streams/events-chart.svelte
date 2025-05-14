<script lang="ts">
    import { Chart, type ECMouseEvent } from 'svelte-echarts';

    import { init, use } from 'echarts/core';
    import {
        graphic,
        type CustomSeriesRenderItemAPI,
        type CustomSeriesRenderItemParams,
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
    import { renderItem } from './shape-renderers';

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
    let dataCount = 20;
    let startTime = +new Date();
    let categories = ['categoryA', 'categoryB', 'categoryC'];
    let types = [
        { name: 'JS Heap', color: '#91c7dd' },
        { name: 'Documents', color: '#bd6d6c' },
    ];

    // Generate mock data. This should come from the backend, not sure how formatted it will be though
    // If we're 100% re-using api data.

    // Worst case can use one category with items coded to different heights and colors
    // otherwise can decrease height and increase size of shapes

    categories.forEach(function (category, index) {
        let baseTime = startTime;
        for (let i = 0; i < dataCount; i++) {
            let typeItem = types[Math.round(Math.random() * (types.length - 1))];
            let duration = Math.round(Math.random() * 10000);

            // Asset has more associated data
            const value =
                index === 0
                    ? [index, baseTime, (baseTime += duration), duration]
                    : [index, (baseTime += duration)];
            data.push({
                name: typeItem.name,
                value: value,
                // We should put our styles here eventually to be consistent on tooltips too
                // itemStyle: {
                //     normal: {
                //         color: typeItem.color,
                //     },
                // },
            });
            baseTime += Math.round(Math.random() * 2000);
        }
    });

    console.log(data);

    const option = {
        tooltip: {
            formatter: function (params: any) {
                // TODO update here
                return params.marker + params.name + ': ' + params.value[3] + ' ms';
            },
        },
        grid: {
            left: '3%',
            top: '20%',
            bottom: '3%',
            height: '30%',
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
            show: false,
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
