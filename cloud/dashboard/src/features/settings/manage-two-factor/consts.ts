export const STEP_TO_TITLE = {
    select: "New 2FA method",
    waiting_authn: "Waiting for results...",
    waiting_totp: "2FA Setup",
    success: "2FA device added",
};

export type AuthStepType = keyof typeof STEP_TO_TITLE;

export const DEFAULT_TOTP_AUTH_NAME = "Mobile Authenticator";
export const DEFAULT_WEBAUTHN_AUTH_NAME = "Security Key";

// Query keys
export const WEBAUTHN_LIST_KEY = "webauthn-list";
export const TOTP_LIST_KEY = "totp-list";
