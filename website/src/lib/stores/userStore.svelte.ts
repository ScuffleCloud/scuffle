import type { User } from '$lib/stores/types';

const mockUser: User = {
    // TODO: Implement this
    id: '1',
    name: 'John Doe',
    email: 'test@example.com',
    avatar: 'https://via.placeholder.com/150',
    organizations: [
        {
            id: '1',
            name: 'Organization 1',
            avatar: 'https://via.placeholder.com/150',
        },
        {
            id: '2',
            name: 'Organization 2',
            avatar: 'https://via.placeholder.com/150',
        },
    ],
};

function createUserStore() {
    // State
    let user = $state<User | null>(null);
    let loading = $state(true);
    let error = $state(false);

    // Initialize async function here

    // async function fetchUser(): Promise<User> {
    //     await new Promise((resolve) => setTimeout(resolve, 300));
    //     user = mockUser;
    //     loading = false;
    //     error = false;
    //     return mockUser;
    // }

    // async function doesn't work here
    function fetchUser() {
        user = mockUser;
        loading = false;
        error = false;
    }

    fetchUser();

    // Return the store
    return { user, loading, error, refresh: fetchUser };
}

export const userStore = createUserStore();
