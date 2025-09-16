import { useAuth } from "$lib/auth.svelte";
import { redirect } from "@sveltejs/kit";

export function load() {
    const auth = useAuth();

    if (auth.userSessionToken.state === "authenticated") {
        throw redirect(307, "/settings/user/common");
    }

    throw redirect(307, "/login");
}
