import { createWebauthnCredential } from "$components/settings/createWebAuthn.svelte";
import { usersServiceClient } from "$lib/grpcClient";
import type { MfaCredential } from "$lib/types";
import { withRpcErrorHandling } from "$lib/utils";
import { createMutation, QueryClient, useQueryClient } from "@tanstack/svelte-query";

type UpdateWebauthnNameType = {
    id: string;
    name: string;
};

export function useCreateWebauthnCredential(userId: string | undefined) {
    const queryClient: QueryClient = useQueryClient();

    return createMutation(() => ({
        mutationFn: () =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");
                return await createWebauthnCredential(userId);
            }),
        onSuccess: () => {
            queryClient.invalidateQueries({
                queryKey: ["webauthn-list"],
            });
        },
    }));
}

export function useUpdateWebauthnName(userId: string | undefined) {
    const queryClient: QueryClient = useQueryClient();

    return createMutation<void, Error, UpdateWebauthnNameType>(() => ({
        mutationFn: ({ id, name }: UpdateWebauthnNameType) =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");

                const call = usersServiceClient.updateWebauthnCredential({
                    userId,
                    id,
                    name,
                });

                const status = await call.status;
                if (status.code !== "OK") {
                    throw new Error(status.detail || "Failed to update credential name");
                }
            }),
        onSuccess: (_data, { id, name }) => {
            queryClient.setQueryData<MfaCredential[]>(
                ["webauthn-list"],
                (old) => old?.map(cred => cred.id === id ? { ...cred, name } : cred),
            );
        },
    }));
}

export function useDeleteWebauthnCredential(userId: string | undefined) {
    const queryClient: QueryClient = useQueryClient();

    return createMutation<void, Error, { id: string }>(() => ({
        mutationFn: async ({ id }: { id: string }) =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");
                const call = usersServiceClient.deleteWebauthnCredential({
                    userId,
                    id,
                });

                const status = await call.status;
                if (status.code !== "OK") {
                    throw new Error(status.detail || "Failed to delete credential");
                }
            }),
        onSuccess: (_data, { id }) => {
            queryClient.setQueryData<MfaCredential[]>(
                ["webauthn-list"],
                (old: MfaCredential[] | undefined) => old?.filter(cred => cred.id !== id),
            );
        },
    }));
}
