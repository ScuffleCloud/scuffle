export interface StreamEvent {
    id: string;
    type: 'neutral' | 'success' | 'error' | 'warning';
    text: string;
    timestamp: string;
}
