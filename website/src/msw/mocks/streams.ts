import type { VideoStream } from '$components/streams/types';
import type { ListResponse } from '$lib/types';

export const mockStreamsListResponse: ListResponse<VideoStream> = {
    count: 6,
    next: null,
    previous: null,
    results: [
        {
            id: '8a28e4dbd6...',
            name: 'purple-angry-bottle...',
            status: 'Live',
            created: '2min ago',
        },
        {
            id: '8a28e499d61...',
            name: 'orange-fluffy-chair...',
            status: 'Live',
            created: '1.2.2025',
        },
        {
            id: '8a28e4dbd62...',
            name: 'purple-angry-bottle...',
            status: 'Finished',
            created: '1.1.2025',
        },
        {
            id: '8a28e4dbd63...',
            name: 'purple-angry-bottle...',
            status: 'Finished',
            created: '1.1.2025',
        },
    ],
};

export const mockStreamsDetailResponse: VideoStream = {
    id: '8a28e4dbd6...',
    name: 'purple-angry-bottle...',
    status: 'Live',
    created: '2min ago',
};
