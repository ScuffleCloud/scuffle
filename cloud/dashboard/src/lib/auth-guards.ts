import { browser } from "$app/environment";
import { authState } from "$lib/auth.svelte";
import { type LoadEvent, redirect } from "@sveltejs/kit";

/**
 * Redirects authenticated users away from auth pages (login, password, passkey)
 * to manage in root page.ts routing
 */
export async function redirectIfAuthenticated(event: LoadEvent) {
    if (!browser) return;
    await event.parent();

    const auth = authState();
    if (auth.userSessionToken.state === "authenticated") {
        redirect(303, "/");
    }
}
