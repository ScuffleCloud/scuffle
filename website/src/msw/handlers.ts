import { http, HttpResponse } from 'msw';
import {
    mockStreamsListResponse,
    mockStreamsDetailResponse,
    mockStreamsCreateResponse,
} from './mocks/streams';
import { mockUserResponse } from './mocks/user';

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

    // Just putting user stuff here for now
    http.get('/api/me', () => {
        return HttpResponse.json(mockUserResponse);
    }),
];
