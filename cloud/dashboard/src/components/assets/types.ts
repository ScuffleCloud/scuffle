export type TimelineDotVariant = "default" | "live" | "finished";

type DisplayMode = "GRID" | "LIST";

export const DISPLAY_MODES: Record<DisplayMode, number> = {
    GRID: 0,
    LIST: 1,
};
