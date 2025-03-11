export type NavItem = {
    id: string;
    label: string;
    path: string;
    children: NavItemChild[];
};

export type NavItemChild = {
    id: string;
    label: string;
    path: string;
};
