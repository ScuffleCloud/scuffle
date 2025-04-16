<script lang="ts">
    import { Axis, Chart, Svg } from 'layerchart';
    import { scaleTime } from 'd3-scale';
    import { RANDOM_DATA } from './data';
</script>

<div class="chart-wrapper">
    <Chart
        data={RANDOM_DATA}
        x="date"
        xScale={scaleTime()}
        y="value"
        yDomain={[0, 100]}
        yNice
        padding={{ top: 20, bottom: 40, left: 60, right: 20 }}
    >
        <Svg>
            <Axis placement="bottom" rule />
            <Axis placement="left" rule ticks={5} />
        </Svg>
    </Chart>
</div>

<style>
    .chart-wrapper {
        height: 300px;
        padding: 1rem;
        border: 1px solid #e2e8f0;
        border-radius: 0.5rem;
        /* Add any additional styling you need */
    }

    /* You can add more specific styles for chart elements if needed */
    :global(.axis text) {
        font-size: 12px;
    }

    :global(.axis line) {
        stroke: #e2e8f0;
    }
</style>
