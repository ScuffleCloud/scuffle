export const WEB_AUTHN_INVALID_STATE_ERROR = "An authenticator already exists containing one of the credentials.";
export const WEB_AUTHN_NOT_ALLOWED_ERROR = "WebAuthn operation was cancelled";

/**
 * Reads WebAuthn errors and returns a user-friendly message
 */
export function getWebAuthnErrorMessage(err: unknown): string {
    if (!(err instanceof Error)) {
        return "An unexpected error occurred during authentication.";
    }

    switch (err.name) {
        case "InvalidStateError":
            return WEB_AUTHN_INVALID_STATE_ERROR;
        case "NotAllowedError":
            return WEB_AUTHN_NOT_ALLOWED_ERROR;
        default:
            return "Authentication failed. Please try again.";
    }
}
