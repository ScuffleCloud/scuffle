import { http, HttpResponse } from 'msw';
import {
    mockStreamsListResponse,
    mockStreamsDetailResponse,
    mockStreamsCreateResponse,
    mockStreamEventsOptionsResponse,
    mockStreamEventsOptionsDetailResponse,
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
    // Get all streams in a hosted room
    http.get('/api/v1/video-streams/:id/events', () => {
        return HttpResponse.json(mockStreamEventsOptionsResponse);
    }),
    // Get all events for a stream in a hosted room
    http.get('/api/v1/video-streams/:id/events/:eventId', () => {
        return HttpResponse.json(mockStreamEventsOptionsDetailResponse);
    }),
    // Just putting user stuff here for now
    http.get('/api/me', () => {
        return HttpResponse.json(mockUserResponse);
    }),
];
