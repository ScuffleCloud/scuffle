export const DEFAULT_LOGIN_MODE: LoginMode = "login";

export type LoginMode =
    | "login" // Default login mode - magic-link
    | "password"
    | "passkey"
    | "magic-link-sent"
    | "forgot-password"
    | "password-reset-sent";
