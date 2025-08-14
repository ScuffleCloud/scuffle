<script lang="ts">
    import { Chart, type ECMouseEvent } from 'svelte-echarts';
    import { init, use } from 'echarts/core';
    import { type EChartsOption, type EChartsType } from 'echarts';
    import { MapChart } from 'echarts/charts';
    import {
        GeoComponent,
        TitleComponent,
        TooltipComponent,
        VisualMapComponent,
        ToolboxComponent,
    } from 'echarts/components';
    import { CanvasRenderer } from 'echarts/renderers';
    import { onMount } from 'svelte';
    import { getCssVar } from '$lib/utils';

    use([
        GeoComponent,
        TitleComponent,
        TooltipComponent,
        VisualMapComponent,
        ToolboxComponent,
        MapChart,
        CanvasRenderer,
    ]);

    type Props = {
        mapData?: Array<{ name: string; value: number }> | null;
        theme?: 'light' | 'dark' | object;
        onChartReady?: (chart: EChartsType) => void;
    };

    const { mapData = null, theme = 'light' }: Props = $props();

    // Hong Kong districts data (matching the original example)
    const defaultData = [
        { name: '中西区', value: 20057.34 },
        { name: '湾仔', value: 15477.48 },
        { name: '东区', value: 31686.1 },
        { name: '南区', value: 6992.6 },
        { name: '油尖旺', value: 44045.49 },
        { name: '深水埗', value: 40689.64 },
        { name: '九龙城', value: 37659.78 },
        { name: '黄大仙', value: 45180.97 },
        { name: '观塘', value: 55204.26 },
        { name: '葵青', value: 21900.9 },
        { name: '荃湾', value: 4918.26 },
        { name: '屯门', value: 5881.84 },
        { name: '元朗', value: 4178.01 },
        { name: '北区', value: 2227.92 },
        { name: '大埔', value: 2180.98 },
        { name: '沙田', value: 9172.94 },
        { name: '西贡', value: 3368 },
        { name: '离岛', value: 806.98 },
    ];

    const data = $derived(mapData || defaultData);
    let isLoading = $state(true);

    const nameMap = {
        'Central and Western': '中西区',
        Eastern: '东区',
        Islands: '离岛',
        'Kowloon City': '九龙城',
        'Kwai Tsing': '葵青',
        'Kwun Tong': '观塘',
        North: '北区',
        'Sai Kung': '西贡',
        'Sha Tin': '沙田',
        'Sham Shui Po': '深水埗',
        Southern: '南区',
        'Tai Po': '大埔',
        'Tsuen Wan': '荃湾',
        'Tuen Mun': '屯门',
        'Wan Chai': '湾仔',
        'Wong Tai Sin': '黄大仙',
        'Yau Tsim Mong': '油尖旺',
        'Yuen Long': '元朗',
    };

    const option = $derived({
        title: {
            text: 'Population Density of Hong Kong (2011)',
            subtext: 'Data from Wikipedia',
            textStyle: {
                color: getCssVar('--colors-gray100'),
                fontSize: 18,
                fontWeight: 'bold',
            },
            subtextStyle: {
                color: getCssVar('--colors-gray80'),
                fontSize: 12,
            },
        },
        tooltip: {
            trigger: 'item',
            formatter: '{b}<br/>{c} (p / km2)',
            backgroundColor: getCssVar('--colors-gray10'),
            borderColor: getCssVar('--colors-gray50'),
            textStyle: {
                color: getCssVar('--colors-gray110'),
            },
        },
        toolbox: {
            show: true,
            orient: 'vertical',
            left: 'right',
            top: 'center',
            feature: {
                dataView: { readOnly: false },
                restore: {},
                saveAsImage: {},
            },
            iconStyle: {
                borderColor: getCssVar('--colors-gray80'),
            },
            emphasis: {
                iconStyle: {
                    borderColor: getCssVar('--colors-blue60'),
                },
            },
        },
        visualMap: {
            min: 800,
            max: 50000,
            text: ['High', 'Low'],
            realtime: false,
            calculable: true,
            inRange: {
                color: [
                    getCssVar('--colors-blue10'),
                    getCssVar('--colors-yellow40'),
                    getCssVar('--colors-red60'),
                ],
            },
            textStyle: {
                color: getCssVar('--colors-gray80'),
            },
        },
        series: [
            {
                name: '香港18区人口密度',
                type: 'map',
                map: 'HK',
                label: {
                    show: true,
                    color: getCssVar('--colors-gray90'),
                    fontSize: 10,
                },
                data: data,
                nameMap: nameMap,
                itemStyle: {
                    borderColor: getCssVar('--colors-gray50'),
                    borderWidth: 1,
                },
                emphasis: {
                    itemStyle: {
                        borderColor: getCssVar('--colors-gray70'),
                        borderWidth: 2,
                    },
                    label: {
                        color: getCssVar('--colors-gray110'),
                        fontSize: 12,
                    },
                },
            },
        ],
    });

    let options = $derived({ ...option } as EChartsOption);

    const handleClick = (event: ECMouseEvent) => {
        if (event.data) {
            alert(`${event.name}: ${(event.data as any).value} (p / km2)`);
        } else {
            alert(`${event.name}: No data available`);
        }
    };

    // Load the Hong Kong GeoJSON data from local file
    onMount(async () => {
        try {
            // If you have HK.json in your static folder
            const response = await fetch('/HK.json');
            const geoJson = await response.json();

            // Register the map with ECharts directly
            const { registerMap } = await import('echarts/core');
            registerMap('HK', geoJson);

            isLoading = false;
        } catch (error) {
            console.error('Failed to load Hong Kong map data:', error);
            isLoading = false;
        }
    });
</script>

<div class="map-container">
    {#if isLoading}
        <div class="loading-overlay">
            <div class="loading-spinner"></div>
            <p>Loading Hong Kong map data...</p>
        </div>
    {:else}
        <Chart {init} {options} {theme} onclick={handleClick} />
    {/if}
</div>

<style>
    .map-container {
        position: relative;
        width: 100%;
        height: 600px;
        background-color: var(--colors-gray20);
        border-radius: 0.5rem;
        border: 1px solid var(--colors-gray40);
        overflow: hidden;
    }

    .loading-overlay {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        background-color: var(--colors-gray20);
        color: var(--colors-gray90);
        gap: 1rem;
    }

    .loading-spinner {
        width: 40px;
        height: 40px;
        border: 3px solid var(--colors-gray50);
        border-top: 3px solid var(--colors-blue60);
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        0% {
            transform: rotate(0deg);
        }
        100% {
            transform: rotate(360deg);
        }
    }

    :global(.map-container .echarts-tooltip) {
        border-radius: 0.25rem;
        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
    }
</style>
