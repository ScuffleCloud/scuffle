export const ssr = false;

import type { UserSettings } from "$msw/mocks/settings";
import type { LayoutLoad } from "./$types";

// I'm not sure if we're going to load pages likes this...
// So right now each page will be 4-5 calls depending on what information we need?
// Is that reasonable to do, or we can consolidate get requests another time?

// Some pages also have large amount of content that could be saved at once instead of
// one post request per action changing. Could get messy but guess we can see if that's needed
export const load = (async ({ fetch, depends }) => {
    depends(`settings:user`);

    const fetchSettings = async (): Promise<UserSettings> => {
        const response = await fetch(`/api/v1/users/settings`);
        if (!response.ok) throw new Error(`Failed to fetch settings: ${response.statusText}`);
        return response.json();
    };

    return {
        settings: fetchSettings(),
    };
}) satisfies LayoutLoad;
