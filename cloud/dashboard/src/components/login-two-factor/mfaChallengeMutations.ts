import { withRpcErrorHandling } from "$lib/utils";
import { createMutation, QueryClient, useQueryClient } from "@tanstack/svelte-query";
import { createMfaWebauthnChallenge } from "./createMfaWebauthnChallenge";

export function useCreateWebauthnChallenge(userId: string | undefined) {
    const queryClient: QueryClient = useQueryClient();

    return createMutation(() => ({
        mutationFn: () =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");
                return await createMfaWebauthnChallenge(userId);
            }),
        onSuccess: () => {
            console.log("Webauthn challenge successful");
        },
    }));
}
