import IconEventsAsset from '$lib/images/icon-events-asset.svelte';
import IconEventsError from '$lib/images/icon-events-error.svelte';
import IconEventsNeutral from '$lib/images/icon-events-neutral.svelte';

type EventType = 'info' | 'asset_created' | 'error';

export interface StreamEvent {
    id: string;
    type: EventType;
    text: string;
    timestamp: string;
}

type EventIcon = typeof IconEventsError | typeof IconEventsNeutral | typeof IconEventsAsset;

export const EVENT_ICONS: Record<EventType, EventIcon> = {
    error: IconEventsError,
    asset_created: IconEventsAsset,
    info: IconEventsNeutral,
} as const;
