<script>
    import EventsTab from '$components/streams/events/events-tab.svelte';
    import { page } from '$app/state';

    const { data } = $props();

    const currentEventId = $derived(page.params.eventId);
    const { events, eventDetails } = $derived(data);
</script>

<EventsTab {events} {eventDetails} {currentEventId} />
