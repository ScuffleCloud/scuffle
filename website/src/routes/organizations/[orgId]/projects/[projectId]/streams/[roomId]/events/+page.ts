import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

// TODO: If a webhook connection exists on the streamId we should always redirect to that page instead
export const load = (async ({ fetch, params, parent, url }) => {
    const parentData = await parent();

    // Access the events from the parent layout
    const parentStream = await parentData.stream;
    const events = parentStream.relatedStreams;

    // Only redirect if we're at the root stream page (not already in events route)
    const isRootStreamPage = !url.pathname.includes('/events/');

    // If events exist and we're at root, redirect to the first event
    if (events && events.length > 0 && isRootStreamPage) {
        const topEvent = events[0];
        throw redirect(307, `events/${topEvent.id}`);
    }
    return {};
}) satisfies PageLoad;
