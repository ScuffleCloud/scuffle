import { authState } from "$lib/auth.svelte";

export const ssr = false;

export async function load() {
    await authState().initialize();
}
