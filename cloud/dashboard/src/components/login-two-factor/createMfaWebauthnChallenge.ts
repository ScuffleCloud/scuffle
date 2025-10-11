import { sessionsServiceClient, usersServiceClient } from "$lib/grpcClient";
import { isWebauthnSupported, parseCredentialRequestOptions, serializeCredentialAssertionResponse } from "$lib/utils";

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

    const credential = await navigator.credentials.get({ publicKey }) as PublicKeyCredential | null;

    if (!credential) {
        throw new Error("No credential received from authenticator");
    }

    const responseJson = serializeCredentialAssertionResponse(credential);

    // const userSession = await validateWebauthnMfa(JSON.stringify(credentialResponseData));

    // Returns a usersession that we should just consume locally but we can do that later

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
