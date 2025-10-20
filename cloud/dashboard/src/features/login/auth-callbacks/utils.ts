// Callback function reused in both registration and login magic link routes
export function handleMagicLinkCallback(
    mutateFn: (params: { code: string }) => void,
): void {
    const urlParams = new URLSearchParams(window.location.search);
    const code = urlParams.get("code");

    if (code) {
        mutateFn({ code });
    }
}

export function handleGoogleOAuthCallback(
    mutateFn: (params: { code: string; state: string }) => void,
): void {
    const urlParams = new URLSearchParams(window.location.search);
    const code = urlParams.get("code");
    const state = urlParams.get("state");

    if (code && state) {
        mutateFn({ code, state });
    }
}
