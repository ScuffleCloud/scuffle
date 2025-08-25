import type { User } from '$lib/types';

export const mockUserResponse: User = {
    id: 'usr_123',
    username: 'johndoe',
    email: 'john@example.com',
    avatar_url: 'https://picsum.photos/200',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    organizations: [
        {
            id: 'org_1',
            name: 'Acme Corp',
            slug: 'acme-corp',
            image_url: 'https://picsum.photos/200',
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
            projects: [
                {
                    id: 'proj_1',
                    name: 'Main Website',
                    slug: 'main-website',
                    organization_id: 'org_1',
                    description: 'Company main website project',
                    created_at: '2024-01-01T00:00:00Z',
                    updated_at: '2024-01-01T00:00:00Z',
                },
                {
                    id: 'proj_2',
                    name: 'Mobile App',
                    slug: 'mobile-app',
                    organization_id: 'org_1',
                    description: 'Mobile application development',
                    created_at: '2024-01-02T00:00:00Z',
                    updated_at: '2024-01-02T00:00:00Z',
                },
            ],
        },
        {
            id: 'org_2',
            name: 'Startup Inc',
            slug: 'startup-inc',
            image_url: 'https://picsum.photos/id/237/200/300',
            created_at: '2024-01-02T00:00:00Z',
            updated_at: '2024-01-02T00:00:00Z',
            projects: [
                {
                    id: 'proj_3',
                    name: 'Main Website',
                    slug: 'main-website',
                    organization_id: 'org_1',
                    description: 'Company main website project',
                    created_at: '2024-01-01T00:00:00Z',
                    updated_at: '2024-01-01T00:00:00Z',
                },
            ],
        },
    ],
};
