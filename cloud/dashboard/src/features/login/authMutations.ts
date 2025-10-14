// lib/auth/mutations.ts
import { goto } from "$app/navigation";
import { authState } from "$lib/auth.svelte";
import { sessionsServiceClient } from "$lib/grpcClient";
import { withRpcErrorHandling } from "$lib/utils";
import { createMutation } from "@tanstack/svelte-query";

interface CompleteGoogleLoginParams {
    code: string;
    state: string;
}

export function useInitiateGoogleLogin() {
    return createMutation(() => ({
        mutationFn: () =>
            withRpcErrorHandling(async () => {
                const device = await authState().getDeviceOrInit();
                const response = await sessionsServiceClient.loginWithGoogle({ device }).response;
                window.location.href = response.authorizationUrl;
            }),
    }));
}

export function useCompleteGoogleLogin() {
    return createMutation(() => ({
        mutationFn: ({ code, state }: CompleteGoogleLoginParams) =>
            withRpcErrorHandling(async () => {
                const device = await authState().getDeviceOrInit();
                const response = await sessionsServiceClient.completeLoginWithGoogle({
                    code,
                    state,
                    device,
                }).response;

                if (!response.newUserSessionToken) {
                    throw new Error("No session token received");
                }

                await authState().handleNewUserSessionToken(response.newUserSessionToken);

                if (response.newUserSessionToken?.sessionMfaPending) {
                    goto("/mfa");
                } else {
                    goto("/");
                }
            }),
    }));
}
