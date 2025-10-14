import { redirectIfAuthenticated } from "$lib/auth-guards";
import type { PageLoadEvent } from "./$types";

export async function load(event: PageLoadEvent) {
    await redirectIfAuthenticated(event);
}
