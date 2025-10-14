import { authState } from "$lib/auth.svelte";

export async function load() {
    await authState().initialize();
}
