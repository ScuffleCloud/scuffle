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

// Similarly to above can be converted in the future to use Uint8Array.from
export function base64ToUint8Array(base64: string): Uint8Array {
    const binaryString = atob(base64);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes;
}
