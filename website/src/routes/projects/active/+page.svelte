<script lang="ts">
    import { Area, Axis, Chart, Svg, Tooltip, Highlight } from 'layerchart';
    import { scaleTime } from 'd3-scale';
    import { format as formatDate } from 'date-fns';
    import { RANDOM_DATA } from './data';

    // Calculate start and end times (2 hours before and after)
    // This should be dynamic based on the data maybe but I like the idea of selecting from a range of time
    const firstDate = RANDOM_DATA[0].date;
    const lastDate = RANDOM_DATA[RANDOM_DATA.length - 1].date;

    const startTime = new Date(firstDate);
    startTime.setHours(startTime.getHours());
    startTime.setMinutes(startTime.getMinutes() - 20);

    const endTime = new Date(lastDate);
    endTime.setHours(endTime.getHours());
    endTime.setMinutes(endTime.getMinutes() + 20);

    // Format function for x-axis to show local time
    const formatTime = (date: Date) => {
        return date.toLocaleTimeString([], {
            hour: '2-digit',
            minute: '2-digit',
        });
    };

    // Create time scale with explicit domain
    const timeScale = scaleTime().domain([startTime, endTime]);
</script>

<div class="chart-container">
    <Chart
        data={RANDOM_DATA}
        x="date"
        xScale={timeScale}
        xDomain={[startTime, endTime]}
        y="value"
        yDomain={[0, null]}
        yNice
        padding={{ left: 48, bottom: 64, right: 24, top: 24 }}
        tooltip={{ mode: 'bisect-x' }}
        let:width
    >
        <Svg>
            <Axis placement="left" grid rule />
            <Axis
                placement="bottom"
                format={formatTime}
                labelRotate={-45}
                gridLines={false}
                class="text-sm"
                ticks={(scale: any) => {
                    // Adjust number of ticks based on width
                    const tickCount = Math.max(2, Math.floor(width / 100));
                    return scale.ticks(tickCount);
                }}
                tickLabelProps={{
                    class: 'tick-label',
                }}
            />
            <Area
                data={RANDOM_DATA}
                x="date"
                y="value"
                class="fill-primary/30"
                line={{ class: 'stroke-2 stroke-primary' }}
            />

            <Highlight points lines />
        </Svg>

        <Tooltip.Root let:data>
            <Tooltip.Header>
                {formatDate(data.date, 'HH:mm:ss')}
            </Tooltip.Header>
            <Tooltip.List>
                <Tooltip.Item label="Value" value={data.value} />
            </Tooltip.List>
        </Tooltip.Root>
    </Chart>
</div>

<style>
    .chart-container {
        margin-bottom: 2rem;
        border-radius: 0.5rem;
        overflow: hidden;
        background-color: var(--background-color);
        height: 500px;
        min-width: 800px;
        padding: 0 1rem;
    }

    :global(.tick-label) {
        font-size: 12px;
        y: 10px;
    }
</style>
