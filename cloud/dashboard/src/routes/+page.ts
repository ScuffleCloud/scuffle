import { authState } from "$lib/auth.svelte";
import { redirect } from "@sveltejs/kit";

export function load() {
    if (authState().userSessionToken.state === "authenticated") {
        throw redirect(307, "/projects");
    }

    throw redirect(307, "/login");
}
