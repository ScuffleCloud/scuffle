import { goto } from "$app/navigation";
import { authState } from "$lib/auth.svelte";
import { rpcErrorToString, sessionsServiceClient } from "$lib/grpcClient";
import { base64urlToArrayBuffer } from "$lib/utils";
import { type RpcError } from "@protobuf-ts/runtime-rpc";
import { CaptchaProvider } from "@scufflecloud/proto/scufflecloud/core/v1/common.js";

let isProcessingMagicLink = false;

/**
 * Sends a magic link to the specified email address
 */
async function sendMagicLink(email: string, captchaToken: string): Promise<void> {
    if (!email || !captchaToken) {
        throw new Error("Email and captcha token are required");
    }

    const call = sessionsServiceClient.loginWithMagicLink({
        captcha: {
            provider: CaptchaProvider.TURNSTILE,
            token: captchaToken,
        },
        email,
    });

    try {
        await call.status;
        console.log("Magic link sent successfully to:", email);
    } catch (err) {
        const error = rpcErrorToString(err as RpcError);
        console.error("Magic link failed:", error);
        throw new Error(error);
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

    try {
        const status = await call.status;
        console.log("Magic link completion status:", status);

        const newUserSessionToken = await call.response;
        console.log("Magic link completion response:", newUserSessionToken);

        if (newUserSessionToken) {
            await authState().handleNewUserSessionToken(newUserSessionToken);

            if (newUserSessionToken?.sessionMfaPending) {
                return;
            }

            goto("/");
        } else {
            throw new Error("No session token received");
        }
    } catch (err) {
        throw new Error(rpcErrorToString(err as RpcError));
    }
}

/**
 * Checks URL parameters for magic link callback
 */
async function handleMagicLinkCallback(): Promise<void> {
    // So not affected by $effect in layout
    if (isProcessingMagicLink) return;

    const urlParams = new URLSearchParams(window.location.search);
    const code = urlParams.get("code");

    if (code) {
        isProcessingMagicLink = true;
        await completeMagicLinkLogin(code).catch(console.error);
        isProcessingMagicLink = false;
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
