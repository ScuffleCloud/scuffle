export type NavItem = {
    id: string;
    label: string;
    path: string;
    children?: NavItemChild[];
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    icon?: any;
};

export type NavItemChild = {
    id: string;
    label: string;
    path: string;
};

export type TurnstileError = {
    message: string;
    wasCaptcha: boolean;
};
