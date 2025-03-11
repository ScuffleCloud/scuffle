// Define types
export type VideoStatus = 'Live' | 'Finished';

export interface VideoStream {
    id: string;
    name: string;
    status: VideoStatus;
    created: string;
}
