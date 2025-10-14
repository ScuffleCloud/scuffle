import { authState } from "$lib/auth.svelte";
import { withRpcErrorHandling } from "$lib/utils";
import { createMutation } from "@tanstack/svelte-query";
import { createMfaWebauthnChallenge } from "./createMfaWebauthnChallenge";

export function useCreateWebauthnChallenge(userId: string | undefined) {
    return createMutation(() => ({
        mutationFn: () =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");
                return await createMfaWebauthnChallenge(userId);
            }),
        onSuccess: () => {
            authState().reloadUserForMfa();
        },
    }));
}
