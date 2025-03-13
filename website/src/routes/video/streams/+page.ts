export const ssr = false;

import type { VideoStream } from '$components/streams/types';
import type { PageLoad } from './$types';

export const load = (async ({ depends, fetch }) => {
    depends('streams:data');

    const fetchStreams = async (): Promise<VideoStream[]> => {
        const response = await fetch('https://swapi.dev/api/films/');
        if (!response.ok) throw new Error(`Failed to fetch streams: ${response.statusText}`);
        return response.json();
    };

    return {
        streams: fetchStreams(),
    };
}) satisfies PageLoad;
