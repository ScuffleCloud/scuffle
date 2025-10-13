import { authState } from "$lib/auth.svelte";
import { error, redirect } from "@sveltejs/kit";
import type { LayoutLoadEvent } from "./$types";

// https://svelte.dev/tutorial/kit/route-groups

export async function load({ parent }: LayoutLoadEvent) {
    await parent(); // wait for the root layout to load first

    const auth = authState();
    if (auth.userSessionToken.state === "unauthenticated") {
        redirect(303, "/login");
    } else if (auth.userSessionToken.state === "error") {
        error(500, auth.userSessionToken.error);
    } else if (auth.userSessionToken.state === "authenticated" && !!auth.userSessionToken.data.mfaPending?.length) {
        // If the user has a pending MFA challenge, redirect to the MFA page
        redirect(303, "/mfa");
    }
}
