import {
    circleData,
    diamondData,
    getSampleLineData,
    rectangleData,
} from '$components/streams/events/sample-data';
import type { ChartData, VideoStream } from '$components/streams/types';
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

export const mockStreamEventsOptionsResponse: VideoStream[] = [
    {
        id: '8a28e499d6s7987fd9812937fd981293',
        status: 'live',
        created: 'May 5, 04:01:11',
        name: 'Stream 1',
    },
    {
        id: '3223499d6s7987fd9812123123132123',
        status: 'finished',
        created: 'May 5, 04:01:11',
        name: 'Stream 2',
    },
    {
        id: '21ae213132s7987fd981212312312',
        status: 'finished',
        created: 'May 5, 04:01:11',
        name: 'Stream 3',
    },
    {
        id: '3e3120f5c4e4s7987fd981123123312',
        status: 'finished',
        created: 'May 5, 04:01:11',
        name: 'Stream 4',
    },
];

export const mockStreamEventsOptionsDetailResponse: ChartData = {
    eventData: [...rectangleData(), ...diamondData(), ...circleData()],
    lineData: getSampleLineData(),
};
