/* eslint-disable @typescript-eslint/no-explicit-any */

import { rpcErrorToString } from "$lib/grpcClient";
import type { RpcError } from "@protobuf-ts/runtime-rpc";

// Wrapper to handle RPC errors in mutations
// Support throwing non-rpc errors too
export async function withRpcErrorHandling<T>(fn: () => Promise<T>): Promise<T> {
    try {
        return await fn();
    } catch (err) {
        if (err instanceof Error && !(err as any).meta) {
            throw err;
        }
        throw new Error(rpcErrorToString(err as RpcError));
    }
}

export function isWebauthnSupported(): boolean {
    return !!(navigator.credentials && navigator.credentials.create);
}

export function getCssVar(varName: string): string {
    if (typeof window === "undefined") {
        return "";
    }
    return getComputedStyle(document.documentElement).getPropertyValue(varName).trim();
}

// This can be replaced with Uint8Array.toBase64 in the future
// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array/toBase64
export function arrayBufferToBase64(buffer: ArrayBufferLike): string {
    return btoa(String.fromCharCode(...new Uint8Array(buffer)));
}

// Replace with Uint8Array.fromBase64 in the future
// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array/fromBase64
export function base64ToArrayBuffer(base64: string): ArrayBuffer {
    return Uint8Array.from(atob(base64), c => c.charCodeAt(0)).buffer;
}

export function base64urlToArrayBuffer(base64url: string): ArrayBuffer {
    const padding = "=".repeat((4 - base64url.length % 4) % 4);
    const base64 = base64url.replace(/-/g, "+").replace(/_/g, "/") + padding;

    return base64ToArrayBuffer(base64);
}

export function arrayBufferToBase64url(buffer: ArrayBuffer): string {
    return arrayBufferToBase64(buffer).replace(/=/g, "").replace(/\+/g, "-").replace(
        /\//g,
        "_",
    );
}

/**
 * For creation: Converts server credential options to browser-compatible format
 */
export function parseCredentialCreationOptions(optionsJson: string): PublicKeyCredentialCreationOptions {
    const options = JSON.parse(optionsJson).publicKey;

    return {
        ...options,
        challenge: base64urlToArrayBuffer(options.challenge),
        user: {
            ...options.user,
            id: base64urlToArrayBuffer(options.user.id),
        },
        excludeCredentials: options.excludeCredentials?.map((cred: any) => ({
            ...cred,
            id: base64urlToArrayBuffer(cred.id),
        })) || [],
    };
}

/**
 * For creation: Serializes credential creation response for server
 */
export function serializeCredentialCreationResponse(credential: PublicKeyCredential): string {
    return JSON.stringify({
        id: credential.id,
        rawId: arrayBufferToBase64url(credential.rawId),
        response: {
            attestationObject: arrayBufferToBase64url(
                (credential.response as AuthenticatorAttestationResponse).attestationObject,
            ),
            clientDataJSON: arrayBufferToBase64url(credential.response.clientDataJSON),
        },
        type: credential.type,
        authenticatorAttachment: credential.authenticatorAttachment,
    });
}

/**
 * For challenge: Converts server credential request options to browser-compatible format
 */
export function parseCredentialRequestOptions(optionsJson: string): PublicKeyCredentialRequestOptions {
    const options = JSON.parse(optionsJson).publicKey;
    console.log(options);

    return {
        ...options,
        challenge: base64urlToArrayBuffer(options.challenge),
        allowCredentials: options.allowCredentials?.map((cred: any) => ({
            ...cred,
            id: base64urlToArrayBuffer(cred.id),
        })) || [],
    };
}

/**
 * For challenge: Serializes credential assertion response for server
 */
export function serializeCredentialAssertionResponse(credential: PublicKeyCredential): string {
    const response = credential.response as AuthenticatorAssertionResponse;
    console.log(response);
    return JSON.stringify({
        id: credential.id,
        rawId: arrayBufferToBase64url(credential.rawId),
        response: {
            authenticatorData: arrayBufferToBase64url(response.authenticatorData),
            clientDataJSON: arrayBufferToBase64url(credential.response.clientDataJSON),
            signature: arrayBufferToBase64url(response.signature),
            userHandle: response.userHandle
                ? arrayBufferToBase64url(response.userHandle)
                : null,
        },
        type: credential.type,
        authenticatorAttachment: credential.authenticatorAttachment,
    });
}
