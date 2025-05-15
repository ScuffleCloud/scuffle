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
    import { rectangleData } from './sample-data';
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
    const data = [...rectangleData()];
    let dataCount = 20;
    // let starttime = +new Date()
    const startTime = new Date('2024-01-01T00:00:00').getTime();
    const categories = ['row1', 'row2', 'row3'];

    // Generate mock data. This should come from the backend, not sure how formatted it will be though
    // If we're 100% re-using api data.

    // Worst case can use one category with items coded to different heights and colors
    // otherwise can decrease height and increase size of shapes
    let lineData: { timestamp: number; value: number }[] = [];
    // For rectangle
    categories.forEach(function (category, index) {
        let baseTime = startTime;
        for (let i = 0; i < dataCount; i++) {
            let duration = Math.round(Math.random() * 10000);

            // Asset has more associated data
            // This should probably only be array of length 2 when not a rectangle but TBD to fix this
            const value =
                index === 1
                    ? [index, (baseTime += duration)]
                    : [index, baseTime, (baseTime += duration), duration];
            data.push({
                name: categories[index],
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
    console.log(data);
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
            trigger: 'axis',
            axisPointer: {
                snap: false,
                type: 'line',
                axis: 'x',
                animation: true,
                status: 'show',
            },
            formatter: function (params) {
                const currentTime = params[0].axisValue;

                // Filter all items across all categories that overlap with current time
                const relevantItems = data.filter((item) => {
                    const startTime = item.value[1];
                    const endTime = item.value[2];
                    return currentTime >= startTime && currentTime <= endTime;
                });

                let result = `Time: ${Math.max(0, currentTime - startTime)} ms<br/>`;

                // Sort items by category for consistent display
                relevantItems.sort((a, b) => a.value[0] - b.value[0]);

                relevantItems.forEach((item) => {
                    console.log(item);
                    const duration = item.value[3];
                    const categoryIndex = item.value[0];
                    // result += `<span style="display:inline-block;margin-right:4px;border-radius:10px;width:10px;height:10px;background-color:${
                    //     types.find((t) => t.name === item.name)?.color || '#666'
                    // }"></span>`;
                    result += `${categories[categoryIndex]} - ${item.name}: ${duration}ms<br/>`;
                });

                return result;
            },
            position: function (pos, params, dom, rect, size) {
                return [pos[0], '10%'];
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

<!-- TODO: Add a "updateFilteredData" function to update visible data after zooming so our tooltip filter is less intensive -->
<Chart {init} {options} onclick={handleClick} />

<style>
</style>
