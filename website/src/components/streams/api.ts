import type { VideoStream } from './types';

export async function fetchStreams(): Promise<VideoStream[]> {
    const response = await fetch('https://swapi.dev/api/films/');

    if (!response.ok) throw new Error(`Failed to fetch streams: ${response.statusText}`);
    // return response.json();
    await new Promise((resolve) => setTimeout(resolve, 5000));
    return response.json();
}
