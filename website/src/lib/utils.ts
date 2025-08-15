export function getCssVar(varName: string): string {
    if (typeof window === 'undefined') {
        return '';
    }
    return getComputedStyle(document.documentElement).getPropertyValue(varName).trim();
}
