/**
 * Converts WebAuthn errors to user-friendly messages
 */
export function getWebAuthnErrorMessage(err: unknown): string {
    if (!(err instanceof Error)) {
        return "An unexpected error occurred during authentication.";
    }

    // Can add more but haven't seen more yet
    switch (err.name) {
        case "InvalidStateError":
            return "An authenticator already exists containing one of the credentials.";
        case "NotAllowedError":
            return "Authentication was denied or cancelled.";
        default:
            return "Authentication failed. Please try again.";
    }
}
