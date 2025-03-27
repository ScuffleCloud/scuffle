import { http, HttpResponse } from 'msw';
import { mockStreamsListResponse, mockStreamsDetailResponse } from './mocks/streams';

export const handlers = [
    http.get('/api/v1/video-streams/', () => {
        return HttpResponse.json(mockStreamsListResponse);
    }),
    http.get('/api/v1/video-streams/:id', () => {
        return HttpResponse.json(mockStreamsDetailResponse);
    }),
];
