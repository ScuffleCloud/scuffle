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
