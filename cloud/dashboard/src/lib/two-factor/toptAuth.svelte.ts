import { authState } from "$lib/auth.svelte";
import { usersServiceClient } from "$lib/grpcClient";

// TODO: Fix these functions and rewrite this file. Figure out how we'll bubble up errors and how completing setups
// Will correctly update wherever the list endpoint data is being fetched. This is just testing flow for design
// Don't commit this
async function initiateTotpSetup(userId: string) {
    const createCall = usersServiceClient.createTotpCredential({ id: userId });
    const createStatus = await createCall.status;

    if (createStatus.code !== "OK") {
        throw new Error(createStatus.detail || "Failed to initiate TOTP setup");
    }

    return await createCall.response;
}

async function completeTotpSetup(userId: string, credentialName: string, totpCode: string) {
    const completeCall = usersServiceClient.completeCreateTotpCredential({
        id: userId,
        name: credentialName,
        code: totpCode,
    });

    const completeStatus = await completeCall.status;

    if (completeStatus.code !== "OK") {
        throw new Error(completeStatus.detail || "Failed to complete TOTP setup");
    }

    return await completeCall.response;
}

async function listTotpCredentials(userId: string) {
    const listCall = usersServiceClient.listTotpCredentials({ id: userId });
    const listStatus = await listCall.status;

    if (listStatus.code !== "OK") {
        throw new Error(listStatus.detail || "Failed to list TOTP credentials");
    }

    return await listCall.response;
}

async function deleteTotpCredential(userId: string, credentialId: string) {
    const deleteCall = usersServiceClient.deleteTotpCredential({
        userId,
        id: credentialId,
    });

    const deleteStatus = await deleteCall.status;

    if (deleteStatus.code !== "OK") {
        throw new Error(deleteStatus.detail || "Failed to delete TOTP credential");
    }

    return await deleteCall.response;
}

export interface TotpAuthProps {
    loading: () => boolean;
    error: () => string | null;
    qrCodeData: () => { secretQrcodePng: Uint8Array; secretUrl: string } | null;
    credentials: () => any[] | null;
    recoveryCodes: () => string[] | null;
    initiateTotpSetup: () => Promise<void>;
    completeTotpSetup: (credentialName: string, totpCode: string) => Promise<void>;
    listCredentials: () => Promise<void>;
    deleteCredential: (credentialId: string) => Promise<void>;
}

export function useTotpAuth(): TotpAuthProps {
    let loading = $state(false);
    let error = $state<string | null>(null);
    let qrCodeData = $state<{ secretQrcodePng: Uint8Array; secretUrl: string } | null>(null);
    let credentials = $state<any[] | null>(null);
    let recoveryCodes = $state<string[] | null>(null);

    return {
        loading: () => loading,
        error: () => error,
        qrCodeData: () => qrCodeData,
        credentials: () => credentials,
        recoveryCodes: () => recoveryCodes,

        async initiateTotpSetup() {
            let userId = authState().user?.id;

            if (!userId) {
                error = "User not authenticated";
                return;
            }

            loading = true;
            error = null;

            try {
                const response = await initiateTotpSetup(userId);
                qrCodeData = response;
            } catch (err) {
                error = err instanceof Error ? err.message : "TOTP setup failed";
            } finally {
                loading = false;
            }
        },

        async completeTotpSetup(credentialName: string, totpCode: string) {
            const userId = authState().user?.id;
            if (!userId) {
                error = "User not authenticated";
                return;
            }

            loading = true;
            error = null;

            try {
                await completeTotpSetup(userId, credentialName, totpCode);
                qrCodeData = null; // Clear after successful completion
                await this.listCredentials(); // Refresh list. This will have to move eventually depending on where the data is stored.
            } catch (err) {
                error = err instanceof Error ? err.message : "TOTP setup completion failed";
            } finally {
                loading = false;
            }
        },

        async listCredentials() {
            const userId = authState().user?.id;
            if (!userId) {
                error = "User not authenticated";
                return;
            }

            loading = true;
            error = null;

            try {
                const response = await listTotpCredentials(userId);
                credentials = response.credentials;
            } catch (err) {
                error = err instanceof Error ? err.message : "Failed to load TOTP credentials";
            } finally {
                loading = false;
            }
        },

        async deleteCredential(credentialId: string) {
            const userId = authState().user?.id;
            if (!userId) {
                error = "User not authenticated";
                return;
            }

            loading = true;
            error = null;

            try {
                await deleteTotpCredential(userId, credentialId);
                // Refresh credentials list
                await this.listCredentials();
            } catch (err) {
                error = err instanceof Error ? err.message : "Failed to delete TOTP credential";
            } finally {
                loading = false;
            }
        },
    };
}
