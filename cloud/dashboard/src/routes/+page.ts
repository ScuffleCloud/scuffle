import { authState } from "$lib/auth.svelte";
import { error, redirect } from "@sveltejs/kit";
import type { PageLoadEvent } from "./$types";

export async function load({ parent }: PageLoadEvent) {
    await parent(); // wait for the layout to load first

    const auth = authState();
    if (auth.userSessionToken.state === "authenticated") {
        redirect(303, "/projects");
    } else if (auth.userSessionToken.state === "unauthenticated") {
        redirect(303, "/login");
    } else if (auth.userSessionToken.state === "error") {
        error(500, auth.userSessionToken.error);
    }
}
