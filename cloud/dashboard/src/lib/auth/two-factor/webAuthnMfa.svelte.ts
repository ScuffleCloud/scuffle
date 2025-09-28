import { authState } from "$lib/auth.svelte";
import { sessionsServiceClient, usersServiceClient } from "$lib/grpcClient";
import { arrayBufferToBase64url, base64urlToArrayBuffer } from "$lib/utils";

// Fix these to make sure it works
async function createWebauthnMfaChallenge(userId: string) {
    const challengeCall = usersServiceClient.createWebauthnChallenge({ id: userId });
    const challengeStatus = await challengeCall.status;

    if (challengeStatus.code !== "OK") {
        throw new Error(challengeStatus.detail || "Failed to create WebAuthn challenge");
    }

    return await challengeCall.response;
}

async function validateWebauthnMfa(credentialResponse: string) {
    const validateCall = sessionsServiceClient.validateMfaForUserSession({
        webauthnResponse: credentialResponse,
    });

    const validateStatus = await validateCall.status;

    if (validateStatus.code !== "OK") {
        throw new Error(validateStatus.detail || "Failed to validate WebAuthn credential");
    }

    return await validateCall.response;
}

async function performWebauthnMfaChallenge(userId: string): Promise<void> {
    // Step 1: Create WebAuthn challenge
    const challengeResponse = await createWebauthnMfaChallenge(userId);
    const options = JSON.parse(challengeResponse.optionsJson).publicKey;

    // Step 2: Convert base64url to ArrayBuffer for browser API
    const publicKey: PublicKeyCredentialRequestOptions = {
        ...options,
        challenge: base64urlToArrayBuffer(options.challenge),
        allowCredentials: options.allowCredentials?.map((cred: any) => ({
            ...cred,
            id: base64urlToArrayBuffer(cred.id),
        })) || [],
    };

    // Step 3: Get credential from browser
    const credential = await navigator.credentials.get({ publicKey }) as PublicKeyCredential | null;

    if (!credential) {
        throw new Error("No credential received from authenticator");
    }

    // Step 4: Prepare response for validation
    const credentialResponseData = {
        id: credential.id,
        rawId: arrayBufferToBase64url(credential.rawId),
        response: {
            authenticatorData: arrayBufferToBase64url(
                (credential.response as AuthenticatorAssertionResponse).authenticatorData,
            ),
            clientDataJSON: arrayBufferToBase64url(credential.response.clientDataJSON),
            signature: arrayBufferToBase64url(
                (credential.response as AuthenticatorAssertionResponse).signature,
            ),
        },
        type: credential.type,
    };

    const userSession = await validateWebauthnMfa(JSON.stringify(credentialResponseData));

    if (userSession) {
        // await authState().handleNewUserSessionToken(userSession);
        console.log("user session finished?");
    }
}

export interface WebauthnMfaAuthProps {
    loading: () => boolean;
    error: () => string | null;
    authenticate: () => Promise<void>;
    isSupported: () => boolean;
}

export const useWebauthnMfaAuth = (): WebauthnMfaAuthProps => {
    let loading = $state(false);
    let error = $state<string | null>(null);

    return {
        loading: () => loading,
        error: () => error,
        async authenticate() {
            const userId = authState().user?.id;
            if (!userId) {
                error = "User not authenticated";
                return;
            }

            loading = true;
            error = null;

            try {
                await performWebauthnMfaChallenge(userId);
            } catch (err: any) {
                console.error("WebAuthn MFA failed:", err);
                error = err.message || "WebAuthn authentication failed";
            } finally {
                loading = false;
            }
        },
        isSupported: () => {
            return !!(navigator.credentials && navigator.credentials.get);
        },
    };
};
