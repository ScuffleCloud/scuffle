import { sessionsServiceClient, usersServiceClient } from "$lib/grpcClient";
import { isWebauthnSupported, parseCredentialRequestOptions, serializeCredentialAssertionResponse } from "$lib/utils";
import { getWebAuthnErrorMessage } from "../settings/manage-two-factor/utils";

export async function createMfaWebauthnChallenge(userId: string): Promise<void> {
    if (!isWebauthnSupported()) {
        throw new Error("WebAuthn not supported on this browser");
    }

    const challengeCall = usersServiceClient.createWebauthnChallenge({ id: userId });
    const challengeStatus = await challengeCall.status;

    if (challengeStatus.code !== "OK") {
        throw new Error(challengeStatus.detail || "Failed to initiate WebAuthn credential challenge");
    }

    const challengeResponse = await challengeCall.response;

    const publicKey = parseCredentialRequestOptions(challengeResponse.optionsJson);

    let credential: PublicKeyCredential | null = null;

    try {
        credential = await navigator.credentials.get({ publicKey }) as PublicKeyCredential | null;
    } catch (err) {
        throw new Error(getWebAuthnErrorMessage(err));
    }

    if (!credential) {
        throw new Error("No credential received from authenticator");
    }

    const responseJson = serializeCredentialAssertionResponse(credential);

    console.log("collected credential now validating for session");
    await sessionsServiceClient.validateMfaForUserSession({
        response: {
            oneofKind: "webauthn",
            webauthn: {
                responseJson,
            },
        },
    }).response;
    console.log("completely validation for session");
}
