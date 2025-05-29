export const ssr = false;

import type { StreamEvent, ChartData } from '$components/streams/types';
import type { LayoutLoad } from './$types';

export const load = (async ({ params, fetch, depends }) => {
    const streamId = params.streamId;
    depends(`stream:${streamId}:events`);

    const fetchStreamEvents = async (): Promise<StreamEvent[]> => {
        const response = await fetch(`/api/v1/video-streams/${streamId}/events`);
        if (!response.ok) throw new Error(`Failed to fetch stream events: ${response.status}`);
        return response.json();
    };

    const fetchEventDetails = async (eventId: string): Promise<ChartData> => {
        const response = await fetch(`/api/v1/video-streams/${streamId}/events/${eventId}`);
        if (!response.ok) throw new Error(`Failed to fetch event details: ${response.status}`);
        return response.json();
    };

    // Fetch the list of stream events
    const events = await fetchStreamEvents();

    // Get the first event if it exists, then fetch its details
    let eventDetails: ChartData | null = null;
    if (events.length > 0) {
        const topEvent = events[0];
        eventDetails = await fetchEventDetails(topEvent.id);
    }

    return {
        events,
        eventDetails,
    };
}) satisfies LayoutLoad;
