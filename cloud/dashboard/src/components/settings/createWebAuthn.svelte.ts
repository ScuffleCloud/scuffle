import { usersServiceClient } from "$lib/grpcClient";
import { arrayBufferToBase64url, base64urlToArrayBuffer, isWebauthnSupported } from "$lib/utils";
import { getWebAuthnErrorMessage, WEB_AUTHN_NOT_ALLOWED_ERROR } from "../../lib/two-factor/utils";

export async function createWebauthnCredential(userId: string): Promise<string> {
    if (!isWebauthnSupported()) {
        throw new Error("WebAuthn not supported on this browser");
    }

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

    const completeCall = usersServiceClient.completeCreateWebauthnCredential({
        id: userId,
        responseJson,
    });

    const completeStatus = await completeCall.status;

    if (completeStatus.code !== "OK") {
        throw new Error(completeStatus.detail || "Failed to complete WebAuthn credential creation");
    }

    const completeResponse = await completeCall.response;
    return completeResponse.id;
}
