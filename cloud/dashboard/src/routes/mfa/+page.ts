import { browser } from "$app/environment";
import { authState } from "$lib/auth.svelte";
import { redirect } from "@sveltejs/kit";
import type { PageLoadEvent } from "./$types";

export async function load({ parent }: PageLoadEvent) {
    if (!browser) return;

    await parent(); // wait for the root layout to load first

    const auth = authState();

    const hasPendingMfa = auth.userSessionToken.state === "authenticated"
        && !!auth.userSessionToken.data.mfaPending?.length;

    // This page should only be accessible if the user is authenticated and has a pending MFA challenge
    if (!hasPendingMfa) {
        redirect(303, "/");
    }
}
