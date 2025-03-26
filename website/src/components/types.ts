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
