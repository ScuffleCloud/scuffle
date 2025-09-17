import { authState } from "$lib/auth.svelte";
import { redirect } from "@sveltejs/kit";

// Permission login routes on requiring a logged in user
export function load() {
    if (authState().userSessionToken.state === "authenticated") {
        throw redirect(307, "/projects");
    }

    return {};
}
