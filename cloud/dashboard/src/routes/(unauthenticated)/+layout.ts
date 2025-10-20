import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { authState } from "$lib/auth.svelte";
import type { LayoutLoadEvent } from "./$types";

// https://svelte.dev/tutorial/kit/route-groups

export async function load({ parent }: LayoutLoadEvent) {
    if (!browser) return;

    await parent(); // wait for the root layout to load first

    const auth = authState();

    if (auth.userSessionToken.state === "authenticated" && !auth.userSessionToken.data.mfaPending?.length) {
        goto("/", { replaceState: true });
    }
}
