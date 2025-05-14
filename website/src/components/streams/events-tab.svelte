<!-- This will hold all event stuff for a stream.
 we will have a dropdown selector that is defaulted to the current stream and can show past streams
 There will be an adjacent button to resume or stop live updates
 Then we will have our chart. We need to leverage a charting library to get this wroking properly probably but we will see
  Then there will be a

 -->

<script lang="ts">
    import EventsChart from './events-chart.svelte';
</script>

<div class="events-chart-container">
    <EventsChart />
</div>

<style>
    .events-chart-container {
        height: 250px;
    }
</style>
