export function getCssVar(varName: string): string {
    if (typeof window === "undefined") {
        return "";
    }
    return getComputedStyle(document.documentElement).getPropertyValue(varName).trim();
}

// This can be replaced with Uint8Array.toBase64 in the future
// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array/toBase64
export function arrayBufferToBase64(buffer: ArrayBuffer): string {
    return btoa(String.fromCharCode(...new Uint8Array(buffer)));
}

// Replace with Uint8Array.fromBase64 in the future
// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array/fromBase64
export function base64ToArrayBuffer(base64: string): ArrayBuffer {
    return Uint8Array.from(atob(base64), c => c.charCodeAt(0)).buffer;
}

export function base64urlToArrayBuffer(base64url: string): ArrayBuffer {
    const padding = "=".repeat((4 - base64url.length % 4) % 4);
    const base64 = base64url.replace(/-/g, "+").replace(/_/g, "/") + padding;

    return base64ToArrayBuffer(base64);
}

export function arrayBufferToBase64url(buffer: ArrayBuffer): string {
    return arrayBufferToBase64(buffer).replace(/=/g, "").replace(/\+/g, "-").replace(
        /\//g,
        "_",
    );
}

export function isOAuthCallback(pathname: string, searchParams: URLSearchParams): boolean {
    return searchParams.has("code")
        && (searchParams.has("state") || pathname.includes("magic-link"));
}
