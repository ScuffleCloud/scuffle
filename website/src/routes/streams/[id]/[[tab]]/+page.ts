export const ssr = false;

import type { VideoStream } from '$components/streams/types';
import type { PageLoad } from '../$types';

export const load = (async ({ params, fetch, depends }) => {
    depends('stream:data');

    const streamId = params.id;

    const fetchStream = async (): Promise<VideoStream> => {
        const response = await fetch(`/api/v1/video-streams/${streamId}`);
        if (!response.ok) throw new Error(`Failed to fetch stream: ${response.statusText}`);
        return response.json();
    };

    return {
        stream: fetchStream(),
    };
}) satisfies PageLoad;
