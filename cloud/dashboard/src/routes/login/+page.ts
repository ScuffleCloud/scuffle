import { authState } from "$lib/auth.svelte";
import { redirect } from "@sveltejs/kit";
import type { PageLoadEvent } from "./$types";
import { browser } from "$app/environment";

export async function load({ parent }: PageLoadEvent) {
    if (!browser) return;

    await parent(); // wait for the layout to load first

    // If the user is already logged in, redirect them away from the login page
    const auth = authState();
    if (auth.userSessionToken.state === "authenticated") {
        redirect(303, "/");
    }
}
