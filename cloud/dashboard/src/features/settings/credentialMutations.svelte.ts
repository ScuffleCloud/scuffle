import { usersServiceClient } from "$lib/grpcClient";
import { withRpcErrorHandling } from "$lib/utils";
import { createMutation, QueryClient, useQueryClient } from "@tanstack/svelte-query";
import { createWebauthnCredential } from "./createWebAuthn.svelte";
import type { MfaCredential } from "./types";

type UpdateWebauthnNameType = {
    id: string;
    name: string;
};

// --Webauthn mutations--
export function useCreateWebauthnCredential(userId: string | undefined) {
    return createMutation(() => ({
        mutationFn: () =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");
                return await createWebauthnCredential(userId);
            }),
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
            const existingData = queryClient.getQueryData<MfaCredential[]>(["webauthn-list"]);

            // Check if the credential exists in the cache. They won't yet if the key was just added
            const credentialExists = existingData?.some(cred => cred.id === id);

            if (credentialExists) {
                queryClient.setQueryData<MfaCredential[]>(
                    ["webauthn-list"],
                    (old) => old?.map(cred => cred.id === id ? { ...cred, name } : cred),
                );
            } else {
                // Credential was just added, refetch the list
                queryClient.invalidateQueries({
                    queryKey: ["webauthn-list"],
                });
            }
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

// --TOTP mutations--
export function useCreateTotpCredential(userId: string | undefined) {
    return createMutation(() => ({
        mutationFn: () =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");

                const createCall = usersServiceClient.createTotpCredential({ id: userId });
                const createStatus = await createCall.status;

                if (createStatus.code !== "OK") {
                    throw new Error(createStatus.detail || "Failed to initiate TOTP credential creation");
                }

                const createResponse = await createCall.response;
                return {
                    qrCode: createResponse.secretQrcodePng,
                    secretUrl: createResponse.secretUrl,
                };
            }),
    }));
}

export function useCompleteTotpCredential(userId: string | undefined) {
    return createMutation(() => ({
        mutationFn: ({ code, name }: { code: string; name?: string }) =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");

                const completeCall = usersServiceClient.completeCreateTotpCredential({
                    id: userId,
                    code,
                    name,
                });

                const completeStatus = await completeCall.status;

                if (completeStatus.code !== "OK") {
                    throw new Error(completeStatus.detail || "Failed to complete TOTP credential creation");
                }

                const completeResponse = await completeCall.response;
                return completeResponse;
            }),
    }));
}
