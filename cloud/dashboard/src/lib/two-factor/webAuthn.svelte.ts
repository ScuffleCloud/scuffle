import { authState } from "$lib/auth.svelte";
import { usersServiceClient } from "$lib/grpcClient";
import { arrayBufferToBase64url, base64urlToArrayBuffer } from "$lib/utils";
import { getWebAuthnErrorMessage, WEB_AUTHN_NOT_ALLOWED_ERROR } from "./utils";

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

    // Create credential using browser API
    let credential: PublicKeyCredential | null = null;
    try {
        credential = await navigator.credentials.create({
            publicKey,
        }) as PublicKeyCredential;
    } catch (err) {
        throw new Error(getWebAuthnErrorMessage(err));
    }

    if (!credential) {
        throw new Error(WEB_AUTHN_NOT_ALLOWED_ERROR);
    }

    // Note: Don't spread these properties from credential object
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

    if (completeStatus.code !== "OK") {
        throw new Error(completeStatus.detail || "Failed to complete WebAuthn credential creation");
    }

    // Refetch the webauthn list. Could also optimistically update the list but probably better to just refetch
    // TODO: Refetch the webauthn list
}

export interface WebauthnAuthProps {
    loading: () => boolean;
    error: () => string | null;
    createCredential: (credentialName: string) => Promise<void>;
    isSupported: () => boolean;
}

// TODO: Bubble up thrown errors into toast? or somewhere
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
