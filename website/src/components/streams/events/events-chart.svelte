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

    const interval = 1000;
    const numRecords = 30;

    // Let's attempt to use rich label
    let data: any[] = [];
    let dataCount = 20;
    // let starttime = +new Date()
    const startTime = new Date('2024-01-01T00:00:00').getTime();
    const categories = ['categoryC', 'categoryB', 'categoryA'];
    const types = [
        { name: 'JS Heap', color: '#91c7dd' },
        { name: 'Documents', color: '#bd6d6c' },
    ];

    // Generate mock data. This should come from the backend, not sure how formatted it will be though
    // If we're 100% re-using api data.

    // Worst case can use one category with items coded to different heights and colors
    // otherwise can decrease height and increase size of shapes
    let lineData: { timestamp: number; value: number }[] = [];
    categories.forEach(function (category, index) {
        let baseTime = startTime;
        for (let i = 0; i < dataCount; i++) {
            let typeItem = types[Math.round(Math.random() * (types.length - 1))];
            let duration = Math.round(Math.random() * 10000);

            // Asset has more associated data
            // This should probably only be array of length 2 when not a rectangle but TBD to fix this
            const value = [index, baseTime, (baseTime += duration), duration];
            data.push({
                name: typeItem.name,
                value: value,
            });
            if (index === 0) {
                lineData.push({
                    timestamp: baseTime,
                    value: duration,
                    // We should put our styles here eventually to be consistent on tooltips too
                    // itemStyle: {
                    //     normal: {
                    //         color: typeItem.color,
                    //     },
                    // },
                });
            }
            baseTime += Math.round(Math.random() * 2000);
        }
    });

    const option = {
        grid: [
            {
                left: '1%',
                right: '1%',
                top: '28%',
                height: '25%',
                show: true,
            },
            // {
            //     left: '4.5%',
            //     top: '55%',
            //     height: '30%',
            //     containLabel: true,
            // },
        ],
        xAxis: [
            {
                min: startTime,
                position: 'top',
                gridIndex: 0,
                axisLabel: {
                    formatter: function (val: number) {
                        return Math.max(0, val - startTime) + ' ms';
                    },
                },
            },
            // {
            //     type: 'time',
            //     gridIndex: 1,
            //     show: false,
            //     splitLine: {
            //         show: true,
            //     },
            // },
        ],
        yAxis: [
            {
                data: categories,
                show: false,
            },
            // {
            //     type: 'value',
            //     gridIndex: 1,
            //     show: false,
            // },
        ],
        tooltip: {
            formatter: function (params) {
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
            // {
            //     type: 'line',
            //     data: lineData.map((item) => [item.timestamp, item.value]),
            //     xAxisIndex: 1,
            //     yAxisIndex: 1,
            //     showSymbol: false,
            //     lineStyle: {
            //         color: '#91c7dd',
            //     },
            // },
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
