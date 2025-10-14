export const DEFAULT_TWO_FACTOR_MODE: TwoFactorMode = "webauthn";

export type TwoFactorMode =
    | "webauthn"
    | "totp"
    | "recovery-code";
