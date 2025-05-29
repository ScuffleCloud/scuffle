// Define types
export type VideoStatus = 'live' | 'finished';

export interface VideoStream {
    id: string;
    name: string;
    status: VideoStatus;
    created: string;
}

export interface StreamEvent {
    id: string;
    name: string;
    status: VideoStatus;
    created: string;
}

export type ChartData = {
    eventData: {
        name: string;
        type: string;
        value: number[];
    }[];
    lineData: {
        timestamp: number;
        value: number;
    }[];
};
