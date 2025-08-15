import { browser, dev } from '$app/environment';
import { PUBLIC_VITE_MSW_ENABLED } from '$env/static/public';

// Prevent multiple initializations of MSW
let initialized = false;

export async function enableMocking() {
    if (!dev || !browser || PUBLIC_VITE_MSW_ENABLED !== 'true' || initialized) {
        return Promise.resolve(false);
    }

    try {
        // Dynamically import the worker
        const { worker } = await import('$msw/browser');

        // Start the worker
        await worker.start({
            onUnhandledRequest: 'bypass',
        });

        initialized = true;
        return true;
    } catch (error) {
        console.error('[MSW] Failed to enable mocking:', error);
        return false;
    }
}
