<script lang="ts">
    import { AreaChart, Axis, Chart, Svg } from 'layerchart';
    import { RANDOM_DATA } from './data';
    import { theme } from '$lib/theme';
</script>

<main class="p-10">
    <div class="h-[300px] p-4 border rounded">
        <AreaChart
            data={RANDOM_DATA}
            x="date"
            y="value"
            series={[{ key: 'value', color: theme.colors.rose500 }]}
        />
    </div>
</main>
