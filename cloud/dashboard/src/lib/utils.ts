export function getCssVar(varName: string): string {
    if (typeof window === "undefined") {
        return "";
    }
    return getComputedStyle(document.documentElement).getPropertyValue(varName).trim();
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
