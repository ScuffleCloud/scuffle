export type Streamed<T> = T | Promise<T>;

export type ListResponse<T> = {
    count: number;
    next: string | null;
    previous: string | null;
    results: T[];
};

export interface User {
    id: string;
    username: string;
    email: string;
    avatar_url?: string;
    created_at: string;
    updated_at: string;
    organizations: Organization[];
}

export interface Organization {
    id: string;
    name: string;
    slug: string;
    image_url?: string;
    created_at: string;
    updated_at: string;
    projects: Project[];
}

export interface Project {
    id: string;
    name: string;
    slug: string;
    organization_id: string;
    description?: string;
    created_at: string;
    updated_at: string;
}
