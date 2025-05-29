<script>
    import EventsTab from '$components/streams/events/events-tab.svelte';

    const { data } = $props();

    const { events, eventDetails } = $derived(data);
</script>

<EventsTab {events} {eventDetails} />
