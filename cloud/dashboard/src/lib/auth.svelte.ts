import { browser } from "$app/environment";
import type { Timestamp } from "@scufflecloud/proto/google/protobuf/timestamp.js";
import type { NewUserSessionToken } from "@scufflecloud/proto/scufflecloud/core/v1/sessions_service.js";
import { User } from "@scufflecloud/proto/scufflecloud/core/v1/users.js";
import { arrayBufferToBase64 } from "./utils";

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
    expiresAt: Date | null;
};

function loadUserSessionToken(): AuthState<UserSessionToken> {
    if (!browser) return { state: "loading" };

    const stored = window.localStorage.getItem("userSessionToken");
    if (stored) {
        try {
            const parsedAuth = JSON.parse(stored);
            if (
                !parsedAuth.state || (parsedAuth.state === "authenticated" && !parsedAuth.data)
                || (parsedAuth.state === "error" && !parsedAuth.error)
            ) {
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
         * Generates a new device keypair, persists it and returns the public key part as base64 encoded SPKI.
         */
        async generateNewDeviceKey(): Promise<string> {
            return generateDeviceKeypair().then((keypair) => {
                deviceKeypair = { state: "loaded", data: keypair };
                saveDeviceKey(keypair);
                return window.crypto.subtle.exportKey("spki", keypair.publicKey);
            }).then((spki) => {
                return arrayBufferToBase64(spki);
            });
        },
        async getDevicePublicKeyOrInit(): Promise<string> {
            const key = this.devicePublicKey;
            if (key) {
                return window.crypto.subtle.exportKey("spki", key).then((spki) => {
                    return arrayBufferToBase64(spki);
                });
            } else {
                return this.generateNewDeviceKey();
            }
        },
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
                            expiresAt: newToken.expiresAt ? timestampToDate(newToken.expiresAt) : null,
                        },
                    };

                    userSessionToken = newUserSessionToken;
                    // Persist session token to localStorage on change
                    window.localStorage.setItem("userSessionToken", JSON.stringify(newUserSessionToken));
                    user = loadUser(newUserSessionToken);
                },
            );
        },
        get devicePublicKey(): CryptoKey | null {
            if (!deviceKeypair) return null;
            if (deviceKeypair.state !== "loaded") return null;
            return deviceKeypair.data?.publicKey ?? null;
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
        state: "loading",
    };
}
