import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { authState } from "$lib/auth.svelte";
import { error } from "@sveltejs/kit";
import type { PageLoadEvent } from "./$types";

export async function load({ parent }: PageLoadEvent) {
    if (!browser) return;

    await parent(); // wait for the layout to load first

    const auth = authState();
    if (auth.userSessionToken.state === "authenticated") {
        if (auth.userSessionToken.data.mfaPending?.length) {
            goto("/mfa", { replaceState: true });
        } else {
            goto("/projects", { replaceState: true });
        }
    } else if (auth.userSessionToken.state === "unauthenticated") {
        goto("/login", { replaceState: true });
    } else if (auth.userSessionToken.state === "error") {
        error(500, auth.userSessionToken.error);
    }
}
