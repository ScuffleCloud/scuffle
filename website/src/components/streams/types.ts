// Define types
export type VideoStatus = 'live' | 'finished';

export interface VideoStream {
    id: string;
    name: string;
    status: VideoStatus;
    created: string;
}
