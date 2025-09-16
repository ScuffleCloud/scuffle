// import { useAuth } from "$lib/auth.svelte";
// import { redirect } from "@sveltejs/kit";

// export function load() {
//     const auth = useAuth();

//     // If authenticated, redirect to the dashboard
//     if (auth.userSessionToken.state === "authenticated") {
//         throw redirect(307, "/settings/user/common");
//     }

//     if (auth.userSessionToken.state === "unauthenticated") {
//         throw redirect(307, "/login");
//     }

//     return {};
// }
