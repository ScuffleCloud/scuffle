import { usersServiceClient } from "$lib/grpcClient";
import { withRpcErrorHandling } from "$lib/utils";
import { createMutation, QueryClient, useQueryClient } from "@tanstack/svelte-query";
import { TOTP_LIST_KEY, WEBAUTHN_LIST_KEY } from "./consts";
import { createWebauthnCredential } from "./createWebAuthn.svelte";
import type { MfaCredential, MfaCredentialType } from "./types";

type UpdateCredentialNameType = {
    id: string;
    name: string;
    type: MfaCredentialType;
};

type DeleteCredentialType = {
    id: string;
    type: MfaCredentialType;
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

// --Credential name mutations--
export function useUpdateCredentialName(userId: string | undefined) {
    const queryClient: QueryClient = useQueryClient();

    return createMutation<void, Error, UpdateCredentialNameType>(() => ({
        mutationFn: ({ id, name, type }: UpdateCredentialNameType) =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");

                const updateFunction = type === "webauthn"
                    ? usersServiceClient.updateWebauthnCredential
                    : usersServiceClient.updateTotpCredential;

                const call = updateFunction({ userId, id, name });

                const status = await call.status;
                if (status.code !== "OK") {
                    throw new Error(status.detail || "Failed to update credential name");
                }
            }),
        onSuccess: (_data, { id, name, type }) => {
            const queryKey = type === "webauthn" ? WEBAUTHN_LIST_KEY : TOTP_LIST_KEY;
            const existingData = queryClient.getQueryData<MfaCredential[]>([queryKey]);

            // Check if the credential exists in the cache. They won't yet if the key was just added
            const credentialExists = existingData?.some(cred => cred.id === id);

            if (credentialExists) {
                queryClient.setQueryData<MfaCredential[]>(
                    [queryKey],
                    (old) => old?.map(cred => cred.id === id ? { ...cred, name } : cred),
                );
            } else {
                // Credential was just added, refetch the list
                queryClient.invalidateQueries({
                    queryKey: [queryKey],
                });
            }
        },
    }));
}

export function useDeleteCredential(userId: string | undefined) {
    const queryClient: QueryClient = useQueryClient();

    return createMutation<void, Error, DeleteCredentialType>(() => ({
        mutationFn: async ({ id, type }: DeleteCredentialType) =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");

                const deleteFunction = type === "webauthn"
                    ? usersServiceClient.deleteWebauthnCredential
                    : usersServiceClient.deleteTotpCredential;

                const call = deleteFunction({ userId, id });

                const status = await call.status;
                if (status.code !== "OK") {
                    throw new Error(status.detail || "Failed to delete credential");
                }
            }),
        onSuccess: (_data, { id, type }) => {
            const queryKey = type === "webauthn" ? WEBAUTHN_LIST_KEY : TOTP_LIST_KEY;
            queryClient.setQueryData<MfaCredential[]>(
                [queryKey],
                (old: MfaCredential[] | undefined) => old?.filter(cred => cred.id !== id),
            );
        },
    }));
}
