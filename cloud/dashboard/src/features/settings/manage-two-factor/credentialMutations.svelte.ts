import { queryKeys } from "$lib/consts";
import { usersServiceClient } from "$lib/grpcClient";
import { arrayBufferToBase64, withRpcErrorHandling } from "$lib/utils";
import { createMutation, QueryClient, useQueryClient } from "@tanstack/svelte-query";
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

export type CreateTotpCredentialMutationResponse = {
    qrCodeUrl: string;
    secretKey: string;
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
    return createMutation<CreateTotpCredentialMutationResponse, Error, void>(() => ({
        mutationFn: () =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");

                const createCall = usersServiceClient.createTotpCredential({ id: userId });
                const createStatus = await createCall.status;

                if (createStatus.code !== "OK") {
                    throw new Error(createStatus.detail || "Failed to initiate TOTP credential creation");
                }

                const createResponse = await createCall.response;

                const qrCodeUrl = `data:image/png;base64,${
                    arrayBufferToBase64(new Uint8Array(createResponse.secretQrcodePng).buffer)
                }`;

                const secretKey = createResponse.secretUrl.split("secret=")[1]?.split("&")[0]
                    || createResponse.secretUrl;

                return {
                    qrCodeUrl,
                    secretKey,
                };
            }),
    }));
}

export function useCompleteTotpCredential(userId: string | undefined) {
    return createMutation(() => ({
        mutationFn: ({ code }: { code: string }) =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");

                const completeCall = usersServiceClient.completeCreateTotpCredential({
                    id: userId,
                    code,
                });

                const completeStatus = await completeCall.status;

                if (completeStatus.code !== "OK") {
                    throw new Error(completeStatus.detail || "Failed to complete TOTP credential creation");
                }

                return await completeCall.response;
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

                const call = type === "webauthn"
                    ? usersServiceClient.updateWebauthnCredential({ userId, id, name })
                    : usersServiceClient.updateTotpCredential({ userId, id, name });

                const status = await call.status;
                if (status.code !== "OK") {
                    throw new Error(status.detail || "Failed to update credential name");
                }
            }),
        onSuccess: (_data, { id, name, type }) => {
            const queryKey = type === "webauthn" ? queryKeys.webauthn(userId!) : queryKeys.totp(userId!);
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

                const call = type === "webauthn"
                    ? usersServiceClient.deleteWebauthnCredential({ userId, id })
                    : usersServiceClient.deleteTotpCredential({ userId, id });

                const status = await call.status;
                if (status.code !== "OK") {
                    throw new Error(status.detail || "Failed to delete credential");
                }
            }),
        onSuccess: (_data, { id, type }) => {
            const queryKey = type === "webauthn" ? queryKeys.webauthn(userId!) : queryKeys.totp(userId!);
            queryClient.setQueryData<MfaCredential[]>(
                [queryKey],
                (old: MfaCredential[] | undefined) => old?.filter(cred => cred.id !== id),
            );
        },
    }));
}
