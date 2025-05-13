import type { VideoStream } from '$components/streams/types';
import type { ListResponse } from '$lib/types';

export const mockStreamsListResponse: ListResponse<VideoStream> = {
    count: 10,
    next: null,
    previous: null,
    results: [
        {
            id: '8a28e4dbd6...',
            name: 'purple-angry-bottle...',
            status: 'live',
            created: '2min ago',
        },
        {
            id: '8a28e499d61...',
            name: 'orange-fluffy-chair...',
            status: 'live',
            created: '1.2.2025',
        },
        {
            id: '8a28e4dbd62...',
            name: 'purple-angry-bottle...',
            status: 'finished',
            created: '1.1.2025',
        },
        {
            id: '8a28e4dbd63...',
            name: 'purple-angry-bottle...',
            status: 'finished',
            created: '1.1.2025',
        },
        {
            id: '8a28e4dbd65',
            name: 'red-excited-chair-05',
            status: 'finished',
            created: '1.1.2025',
        },
        {
            id: '8a28e4dbd66',
            name: 'yellow-peaceful-table-06',
            status: 'finished',
            created: '1.1.2025',
        },
        {
            id: '8a28e4dbd67',
            name: 'pink-serene-couch-07',
            status: 'finished',
            created: '1.1.2025',
        },
        {
            id: '8a28e4dbd68',
            name: 'black-quiet-desk-08',
            status: 'finished',
            created: '1.1.2025',
        },
        {
            id: '8a28e4dbd69',
            name: 'white-gentle-lamp-09',
            status: 'finished',
            created: '1.1.2025',
        },
        {
            id: '8a28e4dbd70',
            name: 'brown-steady-chair-10',
            status: 'finished',
            created: '1.1.2025',
        },
        {
            id: '8a28e4dbd62',
            name: 'orange-fluffy-chair-02',
            status: 'finished',
            created: '1.1.2025',
        },
        {
            id: '8a28e4dbd63',
            name: 'green-happy-desk-03',
            status: 'finished',
            created: '1.1.2025',
        },
    ],
};

export const mockStreamsDetailResponse: VideoStream = {
    id: '8a28e4dbd6...',
    name: 'purple-angry-bottle...',
    status: 'live',
    created: '2min ago',
};

export const mockStreamsCreateResponse = {
    status: 200,
    newId: '8a28e4dbd6',
};
