import { goto } from "$app/navigation";
import { authState } from "$lib/auth.svelte";
import { sessionsServiceClient } from "$lib/grpcClient";
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
        onSuccess: async () => {
            await authState().reloadUserForMfa();
            goto("/");
        },
    }));
}

export function useValidateMfaTotp() {
    return createMutation(() => ({
        mutationFn: (totpCode: string) =>
            withRpcErrorHandling(async () => {
                if (!totpCode || totpCode.trim().length === 0) {
                    throw new Error("TOTP code is required");
                }

                // Validate format
                if (!/^\d{6}$/.test(totpCode.trim())) {
                    throw new Error("Invalid TOTP code format");
                }

                await sessionsServiceClient.validateMfaForUserSession({
                    response: {
                        oneofKind: "totp",
                        totp: {
                            code: totpCode.trim(),
                        },
                    },
                }).response;
            }),
        onSuccess: () => {
            authState().reloadUserForMfa();
            goto("/");
        },
    }));
}

export function useValidateMfaRecoveryCode() {
    return createMutation(() => ({
        mutationFn: (recoveryCode: string) =>
            withRpcErrorHandling(async () => {
                if (!recoveryCode || recoveryCode.trim().length === 0) {
                    throw new Error("Recovery code is required");
                }
                await sessionsServiceClient.validateMfaForUserSession({
                    response: {
                        oneofKind: "recoveryCode",
                        recoveryCode: {
                            code: recoveryCode.trim(),
                        },
                    },
                }).response;
            }),
    }));
}
