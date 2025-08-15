import { enableMocking } from '$msw/setup';
import type { Handle } from '@sveltejs/kit';

const mockingPromise = enableMocking();

// Export handle client hook to run before page load functions
export const handleClient: Handle = async ({ event, resolve }) => {
    await mockingPromise;
    return resolve(event);
};
