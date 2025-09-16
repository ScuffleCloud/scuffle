import { useAuth } from "$lib/auth.svelte";
import { redirect } from "@sveltejs/kit";

export function load() {
    const auth = useAuth();

    // This should say dashboard after we add one
    if (auth.userSessionToken.state === "authenticated") {
        throw redirect(307, "/projects");
    }

    throw redirect(307, "/login");
}
