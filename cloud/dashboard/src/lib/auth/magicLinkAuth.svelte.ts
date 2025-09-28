import { goto } from "$app/navigation";
import { authState } from "$lib/auth.svelte";
import { LANDING_ROUTE } from "$lib/consts";
import { sessionsServiceClient } from "$lib/grpcClient";
import { base64urlToArrayBuffer } from "$lib/utils";
import { CaptchaProvider } from "@scufflecloud/proto/scufflecloud/core/v1/common.js";

/**
 * Sends a magic link to the specified email address
 */
async function sendMagicLink(email: string, captchaToken: string): Promise<void> {
    if (!email || !captchaToken) {
        throw new Error("Email and captcha token are required");
    }

    try {
        const call = sessionsServiceClient.loginWithMagicLink({
            captcha: {
                provider: CaptchaProvider.TURNSTILE,
                token: captchaToken,
            },
            email,
        });

        const status = await call.status;

        if (status.code === "OK") {
            console.log("Magic link sent successfully to:", email);
        } else {
            console.error("Magic link failed:", status.detail);
            throw new Error(status.detail || "Failed to send magic link");
        }
    } catch (error) {
        console.error("Magic link error:", error);
        throw error;
    }
}

/**
 * Completes the magic link login process
 */
async function completeMagicLinkLogin(code: string): Promise<void> {
    const device = await authState().getDeviceOrInit();
    const codeBuffer = base64urlToArrayBuffer(code);

    const call = sessionsServiceClient.completeLoginWithMagicLink({
        code: new Uint8Array(codeBuffer),
        device,
    });

    const status = await call.status;
    console.log("Magic link completion status:", status);

    if (status.code === "OK") {
        const newUserSessionToken = await call.response;
        console.log("Magic link completion response:", newUserSessionToken);

        if (newUserSessionToken) {
            await authState().handleNewUserSessionToken(newUserSessionToken);

            if (newUserSessionToken?.sessionMfaPending) {
                return;
            }

            goto(LANDING_ROUTE);
        } else {
            throw new Error("No session token received");
        }
    } else {
        throw new Error(status.detail || "Magic link login completion failed");
    }
}

/**
 * Checks URL parameters for magic link callback. Place in $effect.
 */
function handleMagicLinkCallback(): void {
    const urlParams = new URLSearchParams(window.location.search);
    const code = urlParams.get("code");

    if (code) {
        completeMagicLinkLogin(code).catch(console.error);
    }
}

/**
 * Composable for magic link authentication
 */
export function useMagicLinkAuth() {
    return {
        sendMagicLink,
        handleMagicLinkCallback,
    };
}
