import { usersServiceClient } from "$lib/grpcClient";
import { isWebauthnSupported, parseCredentialCreationOptions, serializeCredentialCreationResponse } from "$lib/utils";
import { getWebAuthnErrorMessage, WEB_AUTHN_NOT_ALLOWED_ERROR } from "./utils";

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
    const publicKey = parseCredentialCreationOptions(createResponse.optionsJson);

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

    const responseJson = serializeCredentialCreationResponse(credential);

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
