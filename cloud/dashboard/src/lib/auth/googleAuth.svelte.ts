import { useAuth } from "$lib/auth.svelte";
import { sessionsServiceClient } from "$lib/grpcClient";

/**
 * Initiates Google OAuth login flow
 */
export async function initiateGoogleLogin(): Promise<void> {
    const auth = useAuth();

    const device = await auth.getDeviceOrInit();

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
    const auth = useAuth();

    const device = await auth.getDeviceOrInit();

    // I think this is broken should revisit later?
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

        // if (response.newUserSessionToken?.sessionMfaPending) {
        //     // Redirect to MFA page?
        //     window.location.href = "/mfa";
        //     return;
        // }

        if (response.newUserSessionToken) {
            await auth.handleNewUserSessionToken(response.newUserSessionToken);
        } else {
            throw new Error("No session token received");
        }
    } else {
        throw new Error(status.detail || "Google login completion failed");
    }
}

/**
 * Checks URL parameters for OAuth callback. Place in $effect.
 */
export function handleGoogleOAuthCallback(): void {
    const urlParams = new URLSearchParams(window.location.search);
    const code = urlParams.get("code");
    const state = urlParams.get("state");

    if (code && state) {
        completeGoogleLogin(code, state).catch(console.error);
    }
}

/**
 * Hook for Google authentication with reactive state
 */
export function useGoogleAuth() {
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
