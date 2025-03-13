// Sample mock data
export const mockStreams: VideoStream[] = [
    {
        id: 'mock-1',
        name: 'Test Stream 1',
        status: 'Live',
        created: '2023-01-01',
    },
    {
        id: 'mock-2',
        name: 'Test Stream 2',
        status: 'Finished',
        created: '2023-01-02',
    },
];

import type { VideoStream } from '$components/streams/types';
import { http, HttpResponse } from 'msw';

export const handlers = [
    http.get('https://swapi.dev/api/films/', ({ request, params }) => {
        return HttpResponse.json({
            results: mockStreams.map((stream) => ({
                id: stream.id,
                name: stream.name,
                status: stream.status,
                created: stream.created,
            })),
        });
    }),
];
