import { useAuth } from "$lib/auth.svelte";
import { redirect } from "@sveltejs/kit";

// Permission login routes on requiring a logged in user
export function load() {
    const auth = useAuth();

    if (auth.userSessionToken.state === "authenticated") {
        throw redirect(307, "/projects");
    }

    return {};
}
