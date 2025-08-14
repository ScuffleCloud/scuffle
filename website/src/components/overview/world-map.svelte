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

    // Register the required components
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
        title?: string;
        dataLabel?: string;
    };

    const {
        mapData = null,
        theme = 'light',
        onChartReady,
        title = 'Global Technology Innovation Index 2024',
        dataLabel = 'Innovation Score',
    }: Props = $props();

    // Sample world data - Technology Innovation scores by country
    const defaultData = [
        { name: 'United States', value: 95 },
        { name: 'China', value: 88 },
        { name: 'Japan', value: 85 },
        { name: 'Germany', value: 82 },
        { name: 'South Korea', value: 80 },
        { name: 'United Kingdom', value: 78 },
        { name: 'France', value: 76 },
        { name: 'Canada', value: 74 },
        { name: 'Israel', value: 72 },
        { name: 'Singapore', value: 70 },
        { name: 'Sweden', value: 68 },
        { name: 'Switzerland', value: 66 },
        { name: 'Netherlands', value: 64 },
        { name: 'Finland', value: 62 },
        { name: 'Denmark', value: 60 },
        { name: 'Norway', value: 58 },
        { name: 'Australia', value: 56 },
        { name: 'Belgium', value: 54 },
        { name: 'Austria', value: 52 },
        { name: 'Ireland', value: 50 },
        { name: 'Taiwan', value: 48 },
        { name: 'India', value: 46 },
        { name: 'Italy', value: 44 },
        { name: 'Spain', value: 42 },
        { name: 'Russia', value: 40 },
        { name: 'Brazil', value: 38 },
        { name: 'Mexico', value: 36 },
        { name: 'Poland', value: 34 },
        { name: 'Turkey', value: 32 },
        { name: 'Czech Republic', value: 30 },
        { name: 'Portugal', value: 28 },
        { name: 'Hungary', value: 26 },
        { name: 'Chile', value: 24 },
        { name: 'Greece', value: 22 },
        { name: 'Thailand', value: 20 },
        { name: 'Malaysia', value: 18 },
        { name: 'South Africa', value: 16 },
        { name: 'Argentina', value: 14 },
        { name: 'Colombia', value: 12 },
        { name: 'Philippines', value: 10 },
        { name: 'Indonesia', value: 8 },
        { name: 'Vietnam', value: 6 },
        { name: 'Egypt', value: 4 },
        { name: 'Nigeria', value: 2 },
    ];

    const data = $derived(mapData || defaultData);
    let isLoading = $state(true);
    let chartInstance: EChartsType | null = $state(null);

    const maxValue = $derived(Math.max(...data.map((item) => item.value)));
    const minValue = $derived(Math.min(...data.map((item) => item.value)));

    const option = $derived({
        title: {
            text: title,
            subtext: `Data represents ${dataLabel.toLowerCase()} across different countries`,
            left: 'center',
            top: 20,
            textStyle: {
                color: getCssVar('--colors-gray100'),
                fontSize: 20,
                fontWeight: 'bold',
            },
            subtextStyle: {
                color: getCssVar('--colors-gray80'),
                fontSize: 12,
            },
        },
        tooltip: {
            trigger: 'item',
            formatter: function (params: any) {
                if (params.data) {
                    return `<strong>${params.name}</strong><br/>${dataLabel}: ${params.data.value}`;
                }
                return `<strong>${params.name}</strong><br/>No data available`;
            },
            backgroundColor: getCssVar('--colors-gray10'),
            borderColor: getCssVar('--colors-gray50'),
            textStyle: {
                color: getCssVar('--colors-gray110'),
            },
        },
        // Toolbox on the right to zoom, refresh, copy, download, ect.
        toolbox: {
            show: true,
            orient: 'horizontal',
            left: 'right',
            top: 80,
            feature: {
                dataView: {
                    readOnly: false,
                    title: 'View Data',
                    lang: ['Data View', 'Close', 'Refresh'],
                },
                restore: {
                    title: 'Reset Zoom',
                },
                saveAsImage: {
                    title: 'Save as Image',
                },
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
            min: minValue,
            max: maxValue,
            text: ['High', 'Low'],
            realtime: false,
            calculable: true,
            orient: 'horizontal',
            left: 'center',
            bottom: 30,
            inRange: {
                color: [
                    getCssVar('--colors-blue10'),
                    getCssVar('--colors-blue30'),
                    getCssVar('--colors-blue50'),
                    getCssVar('--colors-blue70'),
                    getCssVar('--colors-blue90'),
                ],
            },
            textStyle: {
                color: getCssVar('--colors-gray80'),
            },
        },
        geo: {
            map: 'world',
            roam: true,
            zoom: 1.2,
            center: [0, 20],
            label: {
                show: false,
                color: getCssVar('--colors-gray90'),
                fontSize: 8,
            },
            itemStyle: {
                borderColor: getCssVar('--colors-gray50'),
                borderWidth: 0.8,
                areaColor: getCssVar('--colors-gray20'),
            },
            emphasis: {
                itemStyle: {
                    borderColor: getCssVar('--colors-gray70'),
                    borderWidth: 2,
                    areaColor: getCssVar('--colors-gray30'),
                },
                label: {
                    show: true,
                    color: getCssVar('--colors-gray110'),
                    fontSize: 10,
                },
            },
        },
        series: [
            {
                name: dataLabel,
                type: 'map',
                geoIndex: 0,
                data: data,
                itemStyle: {
                    borderColor: getCssVar('--colors-gray50'),
                    borderWidth: 0.8,
                },
                emphasis: {
                    itemStyle: {
                        borderColor: getCssVar('--colors-gray70'),
                        borderWidth: 2,
                    },
                },
            },
        ],
    });

    let options = $derived({ ...option } as EChartsOption);

    const handleClick = (event: ECMouseEvent) => {
        if (event.data) {
            alert(`${event.name}\n${dataLabel}: ${event.data.value}`);
        } else {
            alert(`${event.name}\nNo data available`);
        }
    };

    // Load the simplified world GeoJSON data
    onMount(async () => {
        try {
            // Use the simplified world map from CodePen
            const response = await fetch('/world.json');
            const geoJson = await response.json();

            // Register the map with ECharts directly
            const { registerMap } = await import('echarts/core');
            registerMap('world', geoJson);

            isLoading = false;
        } catch (error) {
            console.error('Failed to load world map data:', error);
            isLoading = false;
        }
    });
</script>

<div class="map-container">
    {#if isLoading}
        <div class="loading-overlay">
            <div class="loading-spinner"></div>
            <p>Loading world map data...</p>
        </div>
    {:else}
        <Chart {init} {options} {theme} onclick={handleClick} />
    {/if}
</div>

<style>
    .map-container {
        position: relative;
        width: 100%;
        height: 700px;
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
