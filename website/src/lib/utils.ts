/**
 * Get the value of a CSS variable for use in JS
 * @param varName - The name of the CSS variable
 * @returns The value of the CSS variable
 */
export function getCssVar(varName: string): string {
    if (typeof window === 'undefined') {
        return '';
    }
    return getComputedStyle(document.documentElement).getPropertyValue(varName).trim();
}
