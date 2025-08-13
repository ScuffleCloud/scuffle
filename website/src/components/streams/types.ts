// Define types
export type VideoStatus = 'live' | 'finished';

export const DEFAULT_LOGIN_MODE: LoginMode = 'magic-link';

export type LoginMode =
    | 'magic-link'
    | 'password'
    | 'passkey'
    | 'magic-link-sent'
    | 'forgot-password'
    | 'password-reset-sent';

export interface VideoStream {
    id: string;
    name: string;
    status: VideoStatus;
    created: string;
    relatedStreams?: VideoStream[];
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
