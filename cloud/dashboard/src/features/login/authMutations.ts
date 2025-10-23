// lib/auth/mutations.ts
import { goto } from "$app/navigation";
import { authState } from "$lib/auth.svelte";
import { sessionsServiceClient } from "$lib/grpcClient";
import { base64urlToArrayBuffer, withRpcErrorHandling } from "$lib/utils";
import { CaptchaProvider } from "@scufflecloud/proto/scufflecloud/core/v1/common.js";
import { createMutation } from "@tanstack/svelte-query";

interface CompleteGoogleLoginParams {
    code: string;
    state: string;
}

interface SendMagicLinkParams {
    email: string;
    captchaToken: string;
}

interface CompleteMagicLinkParams {
    code: string;
}

export interface LoginWithEmailAndPasswordParams {
    email: string;
    password: string;
    captchaToken: string;
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

                if (response.newUserSessionToken?.mfaOptions.length) {
                    goto("/mfa");
                } else {
                    goto("/");
                }
            }),
    }));
}

export function useSendMagicLink() {
    return createMutation(() => ({
        mutationFn: ({ email, captchaToken }: SendMagicLinkParams) =>
            withRpcErrorHandling(async () => {
                if (!email || !captchaToken) {
                    throw new Error("Email and captcha token are required");
                }

                await sessionsServiceClient.loginWithMagicLink({
                    captcha: {
                        provider: CaptchaProvider.TURNSTILE,
                        token: captchaToken,
                    },
                    email,
                }).response;

                console.log("Magic link sent successfully to:", email);
            }),
    }));
}

export function useCompleteMagicLink() {
    return createMutation(() => ({
        mutationFn: ({ code }: CompleteMagicLinkParams) =>
            withRpcErrorHandling(async () => {
                const device = await authState().getDeviceOrInit();
                const codeBuffer = base64urlToArrayBuffer(code);

                const response = await sessionsServiceClient.completeLoginWithMagicLink({
                    code: new Uint8Array(codeBuffer),
                    device,
                }).response;

                if (!response) {
                    throw new Error("No session token received");
                }

                await authState().handleNewUserSessionToken(response);

                if (response?.mfaOptions.length) {
                    goto("/mfa", { replaceState: true });
                } else {
                    goto("/", { replaceState: true });
                }
            }),
    }));
}

// //     loginWithEmailAndPassword(input: LoginWithEmailAndPasswordRequest, options?: RpcOptions): UnaryCall<LoginWithEmailAndPasswordRequest, NewUserSessionToken>;
// export interface LoginWithEmailAndPasswordRequest {
//     /**
//      * @generated from protobuf field: scufflecloud.core.v1.CaptchaChallengeResponse captcha = 1
//      */
//     captcha?: CaptchaChallengeResponse;
//     /**
//      * @generated from protobuf field: string email = 2
//      */
//     email: string;
//     /**
//      * @generated from protobuf field: string password = 3
//      */
//     password: string;
//     /**
//      * @generated from protobuf field: scufflecloud.core.v1.Device device = 4
//      */
//     device?: Device;
// }
export function useLoginWithEmailAndPassword() {
    return createMutation(() => ({
        mutationFn: ({ email, password, captchaToken }: LoginWithEmailAndPasswordParams) =>
            withRpcErrorHandling(async () => {
                if (!email || !password || !captchaToken) {
                    throw new Error("All fields are required");
                }

                const device = await authState().getDeviceOrInit();

                const response = await sessionsServiceClient.loginWithEmailAndPassword({
                    captcha: {
                        provider: CaptchaProvider.TURNSTILE,
                        token: captchaToken,
                    },
                    email,
                    password,
                    device,
                }).response;

                if (!response) {
                    throw new Error("No session token received");
                }

                await authState().handleNewUserSessionToken(response);

                if (response?.mfaOptions.length) {
                    goto("/mfa", { replaceState: true });
                } else {
                    goto("/", { replaceState: true });
                }
            }),
    }));
}
