export const ssr = false;

import type { UserSettings } from '$msw/mocks/settings';
import type { LayoutLoad } from './$types';

export const load = (async ({ params, fetch, depends }) => {
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
