import type { TotpCredential, WebauthnCredential } from "@scufflecloud/proto/scufflecloud/core/v1/users_service.js";

export type MfaCredential =
    | (TotpCredential & { type: "totp" })
    | (WebauthnCredential & { type: "webauthn" });

export type MfaCredentialType = "webauthn" | "totp";
