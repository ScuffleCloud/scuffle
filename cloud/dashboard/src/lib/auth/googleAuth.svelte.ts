import { goto } from "$app/navigation";
import { authState } from "$lib/auth.svelte";
import { LANDING_ROUTE } from "$lib/consts";
import { rpcErrorToString, sessionsServiceClient } from "$lib/grpcClient";
import { type RpcError } from "@protobuf-ts/runtime-rpc";

/**
 * Initiates Google OAuth login flow
 */
async function initiateGoogleLogin(): Promise<void> {
    const device = await authState().getDeviceOrInit();

    try {
        const call = sessionsServiceClient.loginWithGoogle({ device });
        const response = await call.response;

        window.location.href = response.authorizationUrl;
    } catch (err) {
        throw new Error(rpcErrorToString(err as RpcError));
    }
}

/**
 * Completes Google OAuth login flow with authorization code
 */
async function completeGoogleLogin(code: string, state: string): Promise<void> {
    const device = await authState().getDeviceOrInit();

    const call = sessionsServiceClient.completeLoginWithGoogle({
        code,
        state,
        device,
    });

    try {
        const response = await call.response;
        console.log("Google completion response:", response);

        if (response.newUserSessionToken) {
            await authState().handleNewUserSessionToken(response.newUserSessionToken);

            if (response.newUserSessionToken?.sessionMfaPending) return;

            goto(LANDING_ROUTE);
        } else {
            throw new Error("No session token received");
        }
    } catch (err) {
        throw new Error(rpcErrorToString(err as RpcError));
    }
}

/**
 * Checks URL parameters for OAuth callback
 */

let isProcessingGoogleOAuth = false;

async function handleGoogleOAuthCallback(): Promise<void> {
    // So not affected by $effect in layout
    if (isProcessingGoogleOAuth) return;

    const urlParams = new URLSearchParams(window.location.search);
    const code = urlParams.get("code");
    const state = urlParams.get("state");

    if (code && state) {
        isProcessingGoogleOAuth = true;
        await completeGoogleLogin(code, state).catch(console.error);
        isProcessingGoogleOAuth = false;
    }
}

export interface GoogleAuthProps {
    loading: () => boolean;
    error: () => string | null;
    initiateLogin: () => Promise<void>;
    handleOAuthCallback: () => void;
}

/**
 * Access Google authentication functions and states
 */
export function useGoogleAuth(): GoogleAuthProps {
    let loading = $state(false);
    let error = $state<string | null>(null);

    return {
        loading: () => loading,
        error: () => error,
        async initiateLogin() {
            loading = true;
            error = null;
            try {
                await initiateGoogleLogin();
            } catch (err) {
                error = err instanceof Error ? err.message : "Google login failed";
                loading = false;
            }
        },

        handleOAuthCallback: () => handleGoogleOAuthCallback(),
    };
}
