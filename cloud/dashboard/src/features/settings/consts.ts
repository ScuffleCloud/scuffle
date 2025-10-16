export const STEP_TO_TITLE = {
    select: "New 2FA method",
    waiting: "Waiting for results...",
    success: "Passkey added",
};

export type AuthStepType = keyof typeof STEP_TO_TITLE;

export const DEFAULT_TOTP_AUTH_NAME = "Mobile Authenticator";
export const DEFAULT_WEBAUTHN_AUTH_NAME = "Security Key";

// Query keys
export const WEBAUTHN_LIST_KEY = "webauthn-list";
export const TOTP_LIST_KEY = "totp-list";
