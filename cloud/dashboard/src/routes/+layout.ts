import { useAuth } from "$lib/auth.svelte";

export const ssr = false;

export function load() {
    const auth = useAuth();

    if (typeof window !== "undefined") {
        auth.initialize();
    }

    return {};
}
