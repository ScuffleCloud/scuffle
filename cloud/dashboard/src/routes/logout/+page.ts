import { useAuth } from "$lib/auth.svelte";
import { redirect } from "@sveltejs/kit";

export const load = async () => {
    const auth = useAuth();

    try {
        await auth.logout();
    } catch (error) {
        console.error("Logout failed:", error);
        localStorage.removeItem("userSessionToken");
    }

    throw redirect(302, "/login");
};
