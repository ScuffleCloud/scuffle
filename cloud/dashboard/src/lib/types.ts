import type { TotpCredential, WebauthnCredential } from "@scufflecloud/proto/scufflecloud/core/v1/users_service.js";

export const DEFAULT_LOGIN_MODE: LoginMode = "login";
export const DEFAULT_TWO_FACTOR_MODE: TwoFactorMode = "webauthn";

export type LoginMode =
    | "login" // Default login mode - magic-link
    | "password"
    | "passkey"
    | "magic-link-sent"
    | "forgot-password"
    | "password-reset-sent";

export type TwoFactorMode =
    | "webauthn"
    | "totp"
    | "recovery-code";

export type MfaCredential =
    | (TotpCredential & { type: "totp" })
    | (WebauthnCredential & { type: "webauthn" });

// TODO: Most of this can be removed later
export type Streamed<T> = T | Promise<T>;

export type ListResponse<T> = {
    count: number;
    next: string | null;
    previous: string | null;
    results: T[];
};
