import { createQuery, useQueryClient } from '@tanstack/svelte-query';
import type { User, Organization, Project } from '$lib/types';
import { derived, get, writable } from 'svelte/store';

// Query keys for cache management
const QUERY_KEYS = {
    user: ['user'],
} as const;

// API functions
async function fetchUserProfile(): Promise<User> {
    const response = await fetch('/api/me');
    if (!response.ok) throw new Error('Failed to fetch user profile');
    return response.json();
}

// async function fetchUserOrganizations(): Promise<Organization[]> {
//     const response = await fetch('/api/organizations');
//     if (!response.ok) throw new Error('Failed to fetch organizations');
//     return response.json();
// }

// async function fetchOrganizationProjects(organizationId: string): Promise<Project[]> {
//     const response = await fetch(`/api/organizations/${organizationId}/projects`);
//     if (!response.ok) throw new Error('Failed to fetch projects');
//     return response.json();
// }

// async function fetchUserOrganizationRoles(): Promise<UserOrganization[]> {
//     const response = await fetch('/api/me/organization-roles');
//     if (!response.ok) throw new Error('Failed to fetch organization roles');
//     return response.json();
// }

export function useUser() {
    const queryClient = useQueryClient();

    // User profile query
    const userQuery = createQuery({
        queryKey: QUERY_KEYS.user,
        queryFn: fetchUserProfile,
    });

    // // Organizations query
    // const organizationsQuery = createQuery({
    //     queryKey: QUERY_KEYS.organizations,
    //     queryFn: fetchUserOrganizations,
    //     enabled: () => {
    //         const userData = get(userQuery);
    //         return !!userData.data;
    //     },
    // });

    // Keep track of current organization and project with writable stores
    const currentOrganizationStore = writable<Organization | null>(null);
    const currentProjectStore = writable<Project | null>(null);

    // Set initial organization and project when user data loads
    userQuery.subscribe(($userQuery) => {
        if ($userQuery.data?.organizations?.length) {
            const firstOrg = $userQuery.data.organizations[0];
            if (!get(currentOrganizationStore)) {
                currentOrganizationStore.set(firstOrg);
            }
            // Set first project of the current organization if available
            if (firstOrg.projects?.length && !get(currentProjectStore)) {
                currentProjectStore.set(firstOrg.projects[0]);
            }
        }
    });

    // Derived store for all organizations
    const organizations = derived(userQuery, ($query) => $query.data?.organizations ?? []);

    // Derived store for all projects across all organizations
    const allProjects = derived(organizations, ($orgs) => $orgs.flatMap((org) => org.projects));

    // Derived store for current organization's projects
    const currentOrgProjects = derived(
        currentOrganizationStore,
        ($currentOrg) => $currentOrg?.projects ?? [],
    );

    function setCurrentOrganization(orgId: string) {
        const org = get(userQuery).data?.organizations.find((o) => o.id === orgId);
        if (org) {
            currentOrganizationStore.set(org);
            // Reset current project to first project of new org if available
            if (org.projects?.length) {
                currentProjectStore.set(org.projects[0]);
            } else {
                currentProjectStore.set(null);
            }
        }
    }

    function setCurrentProject(projectId: string) {
        const project = get(allProjects).find((p) => p.id === projectId);
        if (project) {
            // Set the project's organization as current if it's different
            const projectOrg = get(organizations).find((org) => org.id === project.organization_id);
            if (projectOrg && get(currentOrganizationStore)?.id !== projectOrg.id) {
                currentOrganizationStore.set(projectOrg);
            }
            currentProjectStore.set(project);
        }
    }

    // Reset function
    const reset = async () => {
        await queryClient.invalidateQueries({ queryKey: QUERY_KEYS.user });
        currentOrganizationStore.set(null);
        currentProjectStore.set(null);
    };

    return {
        // Query stores
        userQuery,
        organizations,
        allProjects,
        currentOrgProjects,

        // Current organization and project
        currentOrganization: currentOrganizationStore,
        currentProject: currentProjectStore,

        // Actions
        setCurrentOrganization,
        setCurrentProject,
        reset,
    };
}
