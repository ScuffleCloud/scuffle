import { browser } from "$app/environment";
import type { Timestamp } from "@scufflecloud/proto/google/protobuf/timestamp.js";
import type { NewUserSessionToken } from "@scufflecloud/proto/scufflecloud/core/v1/sessions_service.js";
import { User } from "@scufflecloud/proto/scufflecloud/core/v1/users.js";

function arrayBufferToBase64(buffer: ArrayBuffer): string {
    return btoa(String.fromCharCode(...new Uint8Array(buffer)));
}

function timestampToDate(timestmap: Timestamp): Date | null {
    let seconds = parseInt(timestmap.seconds);
    if (isNaN(seconds)) return null;

    const millis = seconds * 1000 + timestmap.nanos / 1e6;
    return new Date(millis);
}

export type AuthState<T> = {
    state: "authenticated";
    data: T;
} | {
    state: "unauthenticated";
} | {
    state: "loading";
} | {
    state: "error";
    error: string;
};

export type UserSessionToken = {
    id: string;
    token: string;
    expiresAt: Date | null;
};

function loadUserSessionToken(): AuthState<UserSessionToken> {
    if (!browser) return { state: "loading" };

    const stored = window.localStorage.getItem("userSessionToken");
    if (stored) {
        try {
            const parsedAuth = JSON.parse(stored);
            if (!parsedAuth.state || (parsedAuth.state === "authenticated" && !parsedAuth.data) || (parsedAuth.state === "error" && !parsedAuth.error)) {
                throw new Error("invalid sementics");
            }
            return parsedAuth as AuthState<UserSessionToken>;
        } catch (error) {
            console.error("Failed to parse session token from local storage", error);
            return { state: "error", error: "Failed to parse session token" };
        }
    } else {
        return { state: "unauthenticated" };
    }
}

const RSA_OAEP_SHA256_ALGO = {
    name: "RSA-OAEP",
    modulusLength: 4096,
    publicExponent: new Uint8Array([0x01, 0x00, 0x01]),
    hash: "SHA-256",
};

// TODO: use IndexedDB with non-extractable keys

export async function generateDeviceKeypair(): Promise<CryptoKeyPair> {
    if (!browser) return Promise.reject("Not in browser");
    console.log("Generating new device keypair");

    return window.crypto.subtle.generateKey(RSA_OAEP_SHA256_ALGO, false, ["encrypt", "decrypt"]);
}

function openKeystoreTx(mode: IDBTransactionMode, onsuccess: (store: IDBObjectStore) => void) {
    if (!browser) throw new Error("Not in browser");
    const open = window.indexedDB.open("auth", 1);

    open.onupgradeneeded = (ev) => {
        if (ev.oldVersion === 0) {
            const db = open.result;
            db.createObjectStore("keys");
        }
    };

    open.onerror = (ev) => {
        console.error("Failed to open IndexedDB to save device key", ev);
    };

    open.onsuccess = (_ev) => {
        const db = open.result;
        const tx = db.transaction("keys", mode);
        const store = tx.objectStore("keys");

        onsuccess(store);

        tx.oncomplete = () => {
            db.close();
        };
        tx.onerror = (ev) => {
            console.error("Failed to save device key to IndexedDB", ev);
        };
    };
}

async function loadDeviceKeypair(): Promise<CryptoKeyPair | null> {
    if (!browser) return Promise.reject("Not in browser");
    console.log("Loading device key");

    return new Promise((resolve, _reject) => {
        openKeystoreTx("readonly", (store) => {
            const get = store.get("deviceKey");
            get.onsuccess = () => {
                const keys = get.result as CryptoKeyPair | undefined;
                resolve(keys ?? null);
            };
        });
    });
}

function saveDeviceKey(keypair: CryptoKeyPair) {
    if (!browser) return;
    console.log("Saving device key");

    openKeystoreTx("readwrite", (store) => {
        store.put(keypair, "deviceKey");
    });
}

export type DeviceKeypairState = null | { state: "loading" } | { state: "loaded", data: CryptoKeyPair | null };

export function authState() {
    // Private key
    let deviceKeypair = $state<DeviceKeypairState>(null);
    let userSessionToken = $state<AuthState<UserSessionToken>>(loadUserSessionToken());
    let user = $state<AuthState<User>>(loadUser(userSessionToken));

    return {
        initialize() {
            if (!browser) return;

            if (!deviceKeypair) {
                deviceKeypair = { state: "loading" };
                loadDeviceKeypair().then((keypair) => {
                    deviceKeypair = { state: "loaded", data: keypair };
                }).catch((err) => {
                    console.error("Failed to load device key", err);
                    deviceKeypair = { state: "loaded", data: null };
                });
            }
        },
        /**
         * Generates a new device keypair, persists it and returns the public key
         */
        async generateNewDeviceKey(): Promise<CryptoKey> {
            return generateDeviceKeypair().then((keypair) => {
                deviceKeypair = { state: "loaded", data: keypair };
                saveDeviceKey(keypair);
                return keypair.publicKey;
            });
        },
        async handleNewUserSessionToken(newToken: NewUserSessionToken): Promise<void> {
            if (!browser) return;
            if (!deviceKeypair) throw new Error("Device key is not initialized");
            if (deviceKeypair.state !== "loaded") throw new Error("Device key is not loaded");
            if (!deviceKeypair.data) throw new Error("No device key available to decrypt session token");

            // Decrypt the session token with the device key
            const data = new Uint8Array(newToken.encryptedToken).buffer;
            return window.crypto.subtle.decrypt(RSA_OAEP_SHA256_ALGO, deviceKeypair.data.privateKey, data).then((decrypted) => {
                const newUserSessionToken: AuthState<UserSessionToken> = {
                    state: "authenticated",
                    data: {
                        id: newToken.id,
                        token: arrayBufferToBase64(decrypted),
                        expiresAt: newToken.expiresAt ? timestampToDate(newToken.expiresAt) : null,
                    },
                };

                userSessionToken = newUserSessionToken;
                // Persist session token to localStorage on change
                window.localStorage.setItem("userSessionToken", JSON.stringify(newUserSessionToken));
                user = loadUser(newUserSessionToken);
            });
        },
        get userSessionToken() {
            return userSessionToken;
        },
        get userState() {
            return user;
        },
        get user() {
            return user.state === "authenticated" ? user.data : null;
        },
    };
}

function loadUser(state: AuthState<UserSessionToken>): AuthState<User> {
    if (state.state !== "authenticated") {
        return { ...state };
    }

    // usersServiceClient.getUser({
    // });

    return {
        state: "loading"
    };
}

// // Auth initialization - call this once on app startup
// export const initializeAuth = async (): Promise<void> => {
//     if (!browser) return;

//     try {
//         setLoading(true);
//         clearError();

//         // Check for stored auth state
//         const stored = localStorage.getItem("authState");
//         if (stored) {
//             const parsedAuth: StoredAuthState = JSON.parse(stored);
//             if (parsedAuth.isLoggedIn && parsedAuth.user) {
//                 // Verify with your auth service if needed
//                 // const isValid = await yourAuthPackage.verifyToken();
//                 // if (isValid) {
//                 login(parsedAuth.user);
//                 return;
//                 // }
//             }
//         }

//         // No valid auth found
//         logout();
//     } catch (error) {
//         console.error("Auth initialization failed:", error);
//         setError("Failed to initialize authentication");
//         logout();
//     } finally {
//         setLoading(false);
//     }
// };

// // Auth API functions - replace with api calls but these are all that should exist
// export const authAPI = {
//     async sendMagicLink(email: string): Promise<AuthResult> {
//         try {
//             setLoading(true);
//             clearError();
//             setMagicLinkSent(false);

//             // Your auth package magic link would go here
//             // const result = await yourAuthPackage.sendMagicLink(email);

//             console.log("Sending magic link to:", email);
//             // await new Promise((resolve) => setTimeout(resolve, 1500));

//             setMagicLinkSent(true);
//             return {
//                 success: true,
//                 message: "Check your email for a magic link to continue!",
//             };
//         } catch (error) {
//             console.error("Magic link failed:", error);
//             const errorMessage = error instanceof Error
//                 ? error.message
//                 : "Failed to send magic link. Please try again.";
//             setError(errorMessage);
//             return { success: false, error: errorMessage };
//         } finally {
//             setLoading(false);
//         }
//     },

//     async verifyMagicLink(token: string): Promise<AuthResult> {
//         try {
//             setLoading(true);
//             clearError();

//             // Your auth package verify magic link would go here
//             // const result = await yourAuthPackage.verifyMagicLink(token);

//             console.log("Verifying magic link token:", token);
//             await new Promise((resolve) => setTimeout(resolve, 1000));

//             // Mock user data - replace with actual response from your auth service
//             const userData: User = {
//                 id: crypto.randomUUID(),
//                 name: "John Doe", // This would come from your auth service
//                 email: "user@example.com", // This would come from the verified token
//                 avatar: null,
//                 createdAt: new Date(),
//                 updatedAt: new Date(),
//             };

//             login(userData);
//             return { success: true };
//         } catch (error) {
//             console.error("Magic link verification failed:", error);
//             const errorMessage = error instanceof Error ? error.message : "Invalid or expired magic link.";
//             setError(errorMessage);
//             return { success: false, error: errorMessage };
//         } finally {
//             setLoading(false);
//         }
//     },

//     async loginWithGoogle(): Promise<AuthResult> {
//         try {
//             setLoading(true);
//             clearError();

//             // Your auth package Google login would go here
//             // const result = await yourAuthPackage.loginWithGoogle();

//             console.log("Logging in with Google");
//             await new Promise((resolve) => setTimeout(resolve, 1000));

//             const userData: User = {
//                 id: "456",
//                 name: "John Doe",
//                 email: "john@gmail.com",
//                 avatar: "https://example.com/avatar.jpg",
//                 createdAt: new Date(),
//                 updatedAt: new Date(),
//             };

//             login(userData);
//             return { success: true };
//         } catch (error) {
//             console.error("Google login failed:", error);
//             const errorMessage = error instanceof Error ? error.message : "Google login failed. Please try again.";
//             setError(errorMessage);
//             return { success: false, error: errorMessage };
//         } finally {
//             setLoading(false);
//         }
//     },

//     async loginWithPasskey(): Promise<AuthResult> {
//         try {
//             setLoading(true);
//             clearError();

//             // Your auth package passkey login would go here
//             // const result = await yourAuthPackage.loginWithPasskey();

//             console.log("Logging in with passkey");
//             await new Promise((resolve) => setTimeout(resolve, 1000));

//             const userData: User = {
//                 id: "789",
//                 name: "John Doe",
//                 email: "john@example.com",
//                 isAdmin: true,
//                 avatar: null,
//                 createdAt: new Date(),
//                 updatedAt: new Date(),
//             };

//             login(userData);
//             return { success: true };
//         } catch (error) {
//             console.error("Passkey login failed:", error);
//             const errorMessage = error instanceof Error ? error.message : "Passkey login failed. Please try again.";
//             setError(errorMessage);
//             return { success: false, error: errorMessage };
//         } finally {
//             setLoading(false);
//         }
//     },

//     async loginWithPassword(email: string, password: string): Promise<AuthResult> {
//         try {
//             setLoading(true);
//             clearError();

//             // Your auth package passkey login would go here
//             // const result = await yourAuthPackage.loginWithPasskey();

//             console.log("Logging in with passkey");
//             await new Promise((resolve) => setTimeout(resolve, 1000));

//             const userData: User = {
//                 id: "789",
//                 name: "John Doe",
//                 email: "john@example.com",
//                 isAdmin: true,
//                 avatar: null,
//                 createdAt: new Date(),
//                 updatedAt: new Date(),
//             };

//             login(userData);
//             return { success: true };
//         } catch (error) {
//             console.error("Passkey login failed:", error);
//             const errorMessage = error instanceof Error ? error.message : "Passkey login failed. Please try again.";
//             setError(errorMessage);
//             return { success: false, error: errorMessage };
//         } finally {
//             setLoading(false);
//         }
//     },

//     async logoutUser(): Promise<AuthResult> {
//         try {
//             setLoading(true);
//             clearError();

//             // Your auth package logout would go here
//             // await yourAuthPackage.logout();

//             console.log("Logging out");
//             await new Promise((resolve) => setTimeout(resolve, 500));

//             logout();
//             return { success: true };
//         } catch (error) {
//             console.error("Logout failed:", error);
//             const errorMessage = error instanceof Error ? error.message : "Logout failed. Please try again.";
//             setError(errorMessage);
//             return { success: false, error: errorMessage };
//         } finally {
//             setLoading(false);
//         }
//     },

//     async updateProfile(updates: Partial<User>): Promise<AuthResult> {
//         try {
//             setLoading(true);
//             clearError();

//             // Your auth package update profile would go here
//             // const result = await yourAuthPackage.updateProfile(updates);

//             console.log("Updating profile:", updates);
//             await new Promise((resolve) => setTimeout(resolve, 500));

//             updateUser({ ...updates, updatedAt: new Date() });
//             return { success: true };
//         } catch (error) {
//             console.error("Profile update failed:", error);
//             const errorMessage = error instanceof Error ? error.message : "Failed to update profile.";
//             setError(errorMessage);
//             return { success: false, error: errorMessage };
//         } finally {
//             setLoading(false);
//         }
//     },
// };
