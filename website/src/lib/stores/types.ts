export type User = {
    id: string;
    name: string;
    email: string;
    avatar: string;
    organizations: Organization[];
};

export type Organization = {
    id: string;
    name: string;
    avatar: string;
};
