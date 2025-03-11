import { theme, type ThemeColors } from '$lib/theme';

export const getColor = (col: ThemeColors | string) => {
    return col in theme.colors ? theme.colors[col as ThemeColors] : col;
};

export const TURNSTILE_CONTEXT_KEY = Symbol('turnstile-context-key');
