import { authState } from "$lib/auth.svelte";
import { usersServiceClient } from "$lib/grpcClient";
import { arrayBufferToBase64url, base64urlToArrayBuffer } from "$lib/utils";
import { getWebAuthnErrorMessage } from "./utils";

async function createWebauthnCredential(userId: string, credentialName: string): Promise<void> {
    const createCall = usersServiceClient.createWebauthnCredential({ id: userId });
    const createStatus = await createCall.status;

    if (createStatus.code !== "OK") {
        throw new Error(createStatus.detail || "Failed to initiate WebAuthn credential creation");
    }

    const createResponse = await createCall.response;

    const options = JSON.parse(createResponse.optionsJson).publicKey;

    const publicKey: PublicKeyCredentialCreationOptions = {
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

    // TODO: Explore how to get lastpass to trigger on using the passkey
    // Create credential using browser API
    let credential: PublicKeyCredential | null = null;
    try {
        credential = await navigator.credentials.create({
            publicKey,
        }) as PublicKeyCredential;
    } catch (err) {
        console.log("error", err);
        throw new Error(getWebAuthnErrorMessage(err));
    }

    if (!credential) {
        throw new Error("WebAuthn credential creation was cancelled");
    }

    // Note: These properties aren't spreadable from credential object
    const responseJson = JSON.stringify({
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

    // Complete registration on server
    const completeCall = usersServiceClient.completeCreateWebauthnCredential({
        id: userId,
        name: credentialName,
        responseJson,
    });

    const completeStatus = await completeCall.status;
    const completeResponse = await completeCall.response;

    if (completeStatus.code !== "OK") {
        throw new Error(completeStatus.detail || "Failed to complete WebAuthn credential creation");
    }
}

export interface WebauthnAuthProps {
    loading: () => boolean;
    error: () => string | null;
    createCredential: (credentialName: string) => Promise<void>;
    isSupported: () => boolean;
}

export function useWebauthnAuth(): WebauthnAuthProps {
    let loading = $state(false);
    let error = $state<string | null>(null);

    return {
        loading: () => loading,
        error: () => error,

        async createCredential(credentialName: string) {
            const userId = authState().user?.id;
            if (!userId) {
                error = "User not authenticated";
                return;
            }

            loading = true;
            error = null;

            try {
                await createWebauthnCredential(userId, credentialName);
            } catch (err) {
                error = err instanceof Error ? err.message : "WebAuthn credential creation failed";
            } finally {
                loading = false;
            }
        },

        isSupported: () => {
            return !!(navigator.credentials && navigator.credentials.create);
        },
    };
}
