import { browser } from "$app/environment";
import { authState } from "$lib/auth.svelte";
import { redirect } from "@sveltejs/kit";
import type { LayoutLoadEvent } from "./$types";

// https://svelte.dev/tutorial/kit/route-groups

export async function load({ parent }: LayoutLoadEvent) {
    if (!browser) return;

    await parent(); // wait for the root layout to load first

    const auth = authState();
    if (auth.userSessionToken.state === "authenticated") {
        redirect(303, "/");
    }
}
