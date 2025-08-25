export type NavItem = {
    id: string;
    label: string;
    path: string;
    children?: NavItemChild[];
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
