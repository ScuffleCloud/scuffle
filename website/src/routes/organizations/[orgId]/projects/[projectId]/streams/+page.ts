export const ssr = false;

import type { VideoStream } from '$components/streams/types';
import type { ListResponse } from '$lib/types';
import type { PageLoad } from './$types';

export const load = (async ({ depends, fetch }) => {
    depends('streams:data');

    const fetchStreams = async (): Promise<ListResponse<VideoStream>> => {
        // Fill with generic list response
        const response = await fetch('/api/v1/video-streams/');
        if (!response.ok) throw new Error(`Failed to fetch streams: ${response.statusText}`);
        return response.json();
    };

    return {
        streams: fetchStreams(),
    };
}) satisfies PageLoad;
