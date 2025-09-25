import { goto } from "$app/navigation";
import { authState } from "$lib/auth.svelte";
import { AFTER_LOGIN_LANDING_ROUTE } from "$lib/consts";
import { sessionsServiceClient } from "$lib/grpcClient";

/**
 * Initiates Google OAuth login flow
 */
async function initiateGoogleLogin(): Promise<void> {
    const device = await authState().getDeviceOrInit();

    const call = sessionsServiceClient.loginWithGoogle({ device });
    const status = await call.status;

    if (status.code === "OK") {
        const response = await call.response;
        window.location.href = response.authorizationUrl;
    } else {
        throw new Error(status.detail || "Google login failed");
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

    const status = await call.status;
    console.log("Google completion status:", status);

    if (status.code === "OK") {
        const response = await call.response;
        console.log("Google completion response:", response);

        if (response.newUserSessionToken) {
            await authState().handleNewUserSessionToken(response.newUserSessionToken);

            if (response.newUserSessionToken?.sessionMfaPending) return;

            goto(AFTER_LOGIN_LANDING_ROUTE);
        } else {
            throw new Error("No session token received");
        }
    } else {
        throw new Error(status.detail || "Google login completion failed");
    }
}

/**
 * Checks URL parameters for OAuth callback
 */
function handleGoogleOAuthCallback(): void {
    const urlParams = new URLSearchParams(window.location.search);
    const code = urlParams.get("code");
    const state = urlParams.get("state");

    if (code && state) {
        completeGoogleLogin(code, state).catch(console.error);
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
