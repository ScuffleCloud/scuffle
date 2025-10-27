// Query key factories. Most keys require userId to invalidate, some can require more in the future.
export const queryKeys = {
    webauthn: (userId: string) => ["webauthn-list", userId] as const,
    totp: (userId: string) => ["totp-list", userId] as const,
    sessions: (userId: string) => ["sessions", userId] as const,
} as const;
