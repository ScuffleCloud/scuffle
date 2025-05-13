import { http, HttpResponse } from 'msw';
import {
    mockStreamsListResponse,
    mockStreamsDetailResponse,
    mockStreamsCreateResponse,
} from './mocks/streams';

export const handlers = [
    http.get('/api/v1/video-streams/', () => {
        return HttpResponse.json(mockStreamsListResponse);
    }),
    http.get('/api/v1/video-streams/:id', () => {
        return HttpResponse.json(mockStreamsDetailResponse);
    }),
    http.put('/api/v1/video-streams/new', () => {
        return HttpResponse.json(mockStreamsCreateResponse);
    }),
];
