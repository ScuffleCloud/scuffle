import { withRpcErrorHandling } from "$lib/utils";
import { createMutation, QueryClient, useQueryClient } from "@tanstack/svelte-query";

export function useCreateWebauthnCredential(userId: string | undefined) {
    const queryClient: QueryClient = useQueryClient();

    return createMutation(() => ({
        mutationFn: () =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");
                // return await validateWebauthnChallenge(userId);
            }),
        onSuccess: () => {
            queryClient.invalidateQueries({
                queryKey: ["webauthn-list"],
            });
        },
    }));
}
