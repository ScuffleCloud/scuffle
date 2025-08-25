export const ssr = false;
import type { PageLoad } from './$types';
import type { ChartData } from '$components/streams/types';

export const load = (async ({ params, fetch, depends }) => {
    const roomId = params.roomId;
    depends(`stream:${roomId}:events`);

    const fetchEventDetails = async (eventId: string): Promise<ChartData> => {
        const response = await fetch(`/api/v1/video-streams/${roomId}/events/${eventId}`);
        if (!response.ok) throw new Error(`Failed to fetch event details: ${response.status}`);
        return response.json();
    };

    return {
        eventDetails: fetchEventDetails(params.eventId),
    };
}) satisfies PageLoad;
