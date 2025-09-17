import { browser } from "$app/environment";
import type { Timestamp } from "@scufflecloud/proto/google/protobuf/timestamp.js";
import {
    type Device,
    DeviceAlgorithm,
    MfaOption,
    type NewUserSessionToken,
} from "@scufflecloud/proto/scufflecloud/core/v1/sessions_service.js";
import { User } from "@scufflecloud/proto/scufflecloud/core/v1/users.js";
import { sessionsServiceClient, usersServiceClient } from "./grpcClient";
import { arrayBufferToBase64 } from "./utils";

// Replace with Uint8Array.fromBase64 in the future
// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array/fromBase64
function base64ToArrayBuffer(base64: string): ArrayBuffer {
    return Uint8Array.from(atob(base64), c => c.charCodeAt(0)).buffer;
}

function timestampToDate(timestmap: Timestamp): Date | null {
    const seconds = parseInt(timestmap.seconds);
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
    token: ArrayBuffer;
    userId: string;
    expiresAt: Date | null;
    sessionExpiresAt: Date | null;
    mfaPending: MfaOption[] | null;
};

export type StoredUserSessionToken = {
    id: string;
    token: string;
    userId: string;
    expiresAt: string | null;
    sessionExpiresAt: string | null;
    mfaPending: number[] | null;
};

function toStoredUserSessionToken(token: UserSessionToken): StoredUserSessionToken {
    return {
        id: token.id,
        token: arrayBufferToBase64(token.token),
        userId: token.userId,
        expiresAt: token.expiresAt ? token.expiresAt.toISOString() : null,
        sessionExpiresAt: token.sessionExpiresAt ? token.sessionExpiresAt.toISOString() : null,
        mfaPending: token.mfaPending,
    };
}

function fromStoredUserSessionToken(token: StoredUserSessionToken): UserSessionToken {
    return {
        id: token.id,
        token: base64ToArrayBuffer(token.token),
        userId: token.userId,
        expiresAt: token.expiresAt ? new Date(token.expiresAt) : null,
        sessionExpiresAt: token.sessionExpiresAt ? new Date(token.sessionExpiresAt) : null,
        mfaPending: token.mfaPending ?? null,
    };
}

function loadUserSessionToken(): AuthState<UserSessionToken> {
    if (!browser) return { state: "loading" };

    const stored = window.localStorage.getItem("userSessionToken");
    if (stored) {
        try {
            const parsedStoredAuth = JSON.parse(stored) as AuthState<StoredUserSessionToken>;
            if (
                !parsedStoredAuth.state || (parsedStoredAuth.state === "authenticated" && !parsedStoredAuth.data)
                || (parsedStoredAuth.state === "error" && !parsedStoredAuth.error)
            ) {
                throw new Error("invalid sementics");
            }

            let parsedAuth;
            if (parsedStoredAuth.state === "authenticated") {
                parsedAuth = {
                    ...parsedStoredAuth,
                    data: fromStoredUserSessionToken(parsedStoredAuth.data),
                } as AuthState<UserSessionToken>;
            } else {
                parsedAuth = parsedStoredAuth as AuthState<UserSessionToken>;
            }

            return parsedAuth;
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

async function generateDeviceKeypair(): Promise<CryptoKeyPair> {
    if (!browser) return Promise.reject("Not in browser");
    console.log("Generating new device keypair");

    return window.crypto.subtle.generateKey(RSA_OAEP_SHA256_ALGO, true, ["encrypt", "decrypt"]);
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

    open.onsuccess = () => {
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

    return new Promise((resolve) => {
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

export type DeviceKeypairState = null | { state: "loading" } | { state: "loaded"; data: CryptoKeyPair | null };

// Private key
let deviceKeypair = $state<DeviceKeypairState>(null);
let userSessionToken = $state<AuthState<UserSessionToken>>(loadUserSessionToken());
let user = $state<AuthState<User> | null>(null);

export function authState() {
    return {
        /**
         * Call this function to initialize the auth state. This must be called prior to any other function.
         */
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

            if (!user) {
                user = { state: "loading" };
                loadUser(userSessionToken).then((loadedUser) => {
                    user = loadedUser;
                });
            }
        },
        /**
         * Generates a new device, persists it and returns it.
         *
         * You probably want to use `getDeviceOrInit` instead of this function directly.
         */
        async generateNewDevice(): Promise<Device> {
            return generateDeviceKeypair().then((keypair) => {
                deviceKeypair = { state: "loaded", data: keypair };
                saveDeviceKey(keypair);
                return window.crypto.subtle.exportKey("spki", keypair.publicKey);
            }).then((spki) => {
                return {
                    algorithm: DeviceAlgorithm.RSA_OAEP_SHA256,
                    publicKeyData: new Uint8Array(spki),
                };
            });
        },
        /**
         * Returns the device. If no device exists, a new one is generated.
         */
        async getDeviceOrInit(): Promise<Device> {
            const key = this.devicePublicKey;
            if (key) {
                return window.crypto.subtle.exportKey("spki", key).then((spki) => {
                    return {
                        algorithm: DeviceAlgorithm.RSA_OAEP_SHA256,
                        publicKeyData: new Uint8Array(spki),
                    };
                });
            } else {
                return this.generateNewDevice();
            }
        },
        /**
         * Handles a new user session token by decrypting it with the device key and loading the user.
         * Call this function after a successful login or registration request.
         * That means whenever a NewUserSessionToken is returned by the backend service.
         */
        async handleNewUserSessionToken(newToken: NewUserSessionToken): Promise<void> {
            if (!browser) return;
            if (!deviceKeypair) throw new Error("Device key is not initialized");
            if (deviceKeypair.state !== "loaded") throw new Error("Device key is not loaded");
            if (!deviceKeypair.data) throw new Error("No device key available to decrypt session token");

            // Decrypt the session token with the device key
            const data = new Uint8Array(newToken.encryptedToken).buffer;
            return window.crypto.subtle.decrypt(RSA_OAEP_SHA256_ALGO, deviceKeypair.data.privateKey, data).then(
                (decrypted) => {
                    const newUserSessionToken: AuthState<UserSessionToken> = {
                        state: "authenticated",
                        data: {
                            id: newToken.id,
                            token: decrypted,
                            userId: newToken.userId,
                            expiresAt: newToken.expiresAt ? timestampToDate(newToken.expiresAt) : null,
                            sessionExpiresAt: newToken.sessionExpiresAt
                                ? timestampToDate(newToken.sessionExpiresAt)
                                : null,
                            mfaPending: newToken.sessionMfaPending ? newToken.mfaOptions : null,
                        },
                    };

                    userSessionToken = newUserSessionToken;

                    // Persist session token to localStorage on change
                    const stored = { ...newUserSessionToken, data: toStoredUserSessionToken(newUserSessionToken.data) };
                    window.localStorage.setItem("userSessionToken", JSON.stringify(stored));

                    return loadUser(newUserSessionToken);
                },
            ).catch((err) => {
                console.error("Failed to decrypt session token", err);
                throw new Error("Failed to decrypt session token");
            }).then((loadedUser) => {
                user = loadedUser;
            });
        },
        /**
         * Invalidates the current user session token and clears the auth state.
         */
        async logout() {
            if (!browser) return;

            const call = sessionsServiceClient.invalidateUserSession({});
            const status = await call.status;
            if (status.code === "OK") {
                userSessionToken = { state: "unauthenticated" };
                window.localStorage.removeItem("userSessionToken");
                user = { state: "unauthenticated" };
            } else {
                console.error("Failed to logout", status);
                throw new Error("Failed to logout: " + status.detail);
            }
        },
        /**
         * Clears the current user session token and auth state when the session is expired.
         * When only the token is expired but the session is still valid, the token will be refreshed automatically.
         */
        async checkValidity(): Promise<void> {
            if (userSessionToken.state !== "authenticated") {
                return;
            }

            // Check if session is expired
            if (
                userSessionToken.data.sessionExpiresAt
                && Date.now() > userSessionToken.data.sessionExpiresAt.getTime() - 10 * 1000
            ) {
                userSessionToken = { state: "unauthenticated" };
                window.localStorage.removeItem("userSessionToken");
                user = { state: "unauthenticated" };
                return;
            }

            // Check if token is expired
            if (userSessionToken.data.expiresAt && Date.now() > userSessionToken.data.expiresAt.getTime() - 10 * 1000) {
                const call = sessionsServiceClient.refreshUserSession({}, { skipValidityCheck: true });
                const status = await call.status;
                const response = await call.response;

                if (status.code !== "OK") {
                    throw new Error("Failed to refresh session: " + status.detail);
                }
                this.handleNewUserSessionToken(response);
            }
        },
        /**
         * Returns the device public key. If the keypair is not yet loaded or does not exist, null is returned.
         */
        get devicePublicKey(): CryptoKey | null {
            if (!deviceKeypair) return null;
            if (deviceKeypair.state !== "loaded") return null;
            return deviceKeypair.data?.publicKey ?? null;
        },
        /**
         * Returns the current user session token state.
         * If the state is "authenticated", the token can be found in `data`.
         */
        get userSessionToken() {
            return userSessionToken;
        },
        /**
         * Returns the current user state.
         * If the state is "authenticated", the user data can be found in `data`.
         */
        get userState() {
            return user;
        },
        /**
         * Returns the current authenticated user or null if not authenticated.
         */
        get user() {
            if (!user) throw new Error("User not initialized");
            return user.state === "authenticated" ? user.data : null;
        },
    };
}

async function loadUser(state: AuthState<UserSessionToken>): Promise<AuthState<User>> {
    if (state.state !== "authenticated") {
        return { ...state };
    }

    const call = usersServiceClient.getUser({
        id: state.data.userId,
    });
    const status = await call.status;

    if (status.code === "OK") {
        const user = await call.response;
        return { state: "authenticated", data: user };
    } else {
        return { state: "error", error: status.detail };
    }
}
