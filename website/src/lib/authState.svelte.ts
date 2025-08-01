// Mostly generated. Not sure what this will look like in the end. Will probably replace most of these

import { browser } from '$app/environment';

export interface User {
    id: string;
    name: string;
    email: string;
    isAdmin?: boolean;
    avatar?: string | null;
    createdAt?: Date;
    updatedAt?: Date;
}

export interface AuthState {
    user: User | null;
    isLoggedIn: boolean;
    isLoading: boolean;
    error: string | null;
    magicLinkSent: boolean;
}

export interface AuthResult {
    success: boolean;
    error?: string;
    message?: string;
}

export interface StoredAuthState {
    user: User;
    isLoggedIn: boolean;
}

// Global auth state
export const authState = $state<AuthState>({
    user: null,
    isLoggedIn: false,
    isLoading: true,
    error: null,
    magicLinkSent: false,
});

export function getUserDisplayName(): string {
    return authState.user?.name || authState.user?.email || 'Anonymous';
}

export function getIsAdmin(): boolean {
    return authState.user?.isAdmin === true;
}

export function getIsLoggedIn(): boolean {
    return authState.isLoggedIn;
}

export function getUser(): User | null {
    return authState.user;
}

export function getIsLoading(): boolean {
    return authState.isLoading;
}

export function getError(): string | null {
    return authState.error;
}

export function getMagicLinkSent(): boolean {
    return authState.magicLinkSent;
}

// Pure functions that operate on the state
export const setLoading = (loading: boolean): void => {
    authState.isLoading = loading;
};

export const setError = (error: string | null): void => {
    authState.error = error;
};

export const clearError = (): void => {
    authState.error = null;
};

export const setMagicLinkSent = (sent: boolean): void => {
    authState.magicLinkSent = sent;
};

export const login = (userData: User): void => {
    Object.assign(authState, {
        user: userData,
        isLoggedIn: true,
        error: null,
        isLoading: false,
    });

    // Persist to localStorage
    if (browser) {
        const storageData: StoredAuthState = {
            user: userData,
            isLoggedIn: true,
        };
        localStorage.setItem('authState', JSON.stringify(storageData));
    }
};

export const logout = (): void => {
    Object.assign(authState, {
        user: null,
        isLoggedIn: false,
        error: null,
        isLoading: false,
    });

    if (browser) {
        localStorage.removeItem('authState');
    }
};

export const updateUser = (updates: Partial<User>): void => {
    if (authState.user) {
        Object.assign(authState.user, updates);

        // Update localStorage
        if (browser) {
            try {
                const stored = localStorage.getItem('authState');
                if (stored) {
                    const parsedAuth: StoredAuthState = JSON.parse(stored);
                    parsedAuth.user = authState.user;
                    localStorage.setItem('authState', JSON.stringify(parsedAuth));
                }
            } catch (error) {
                console.error('Failed to update localStorage:', error);
            }
        }
    }
};

// Auth initialization - call this once on app startup
export const initializeAuth = async (): Promise<void> => {
    if (!browser) return;

    try {
        setLoading(true);
        clearError();

        // Check for stored auth state
        const stored = localStorage.getItem('authState');
        if (stored) {
            const parsedAuth: StoredAuthState = JSON.parse(stored);
            if (parsedAuth.isLoggedIn && parsedAuth.user) {
                // Verify with your auth service if needed
                // const isValid = await yourAuthPackage.verifyToken();
                // if (isValid) {
                login(parsedAuth.user);
                return;
                // }
            }
        }

        // No valid auth found
        logout();
    } catch (error) {
        console.error('Auth initialization failed:', error);
        setError('Failed to initialize authentication');
        logout();
    } finally {
        setLoading(false);
    }
};

// Auth API functions - replace with api calls but these are all that should exist
export const authAPI = {
    async sendMagicLink(email: string): Promise<AuthResult> {
        try {
            setLoading(true);
            clearError();
            setMagicLinkSent(false);

            // Your auth package magic link would go here
            // const result = await yourAuthPackage.sendMagicLink(email);

            console.log('Sending magic link to:', email);
            // await new Promise((resolve) => setTimeout(resolve, 1500));

            setMagicLinkSent(true);
            return {
                success: true,
                message: 'Check your email for a magic link to continue!',
            };
        } catch (error) {
            console.error('Magic link failed:', error);
            const errorMessage =
                error instanceof Error
                    ? error.message
                    : 'Failed to send magic link. Please try again.';
            setError(errorMessage);
            return { success: false, error: errorMessage };
        } finally {
            setLoading(false);
        }
    },

    async verifyMagicLink(token: string): Promise<AuthResult> {
        try {
            setLoading(true);
            clearError();

            // Your auth package verify magic link would go here
            // const result = await yourAuthPackage.verifyMagicLink(token);

            console.log('Verifying magic link token:', token);
            await new Promise((resolve) => setTimeout(resolve, 1000));

            // Mock user data - replace with actual response from your auth service
            const userData: User = {
                id: crypto.randomUUID(),
                name: 'John Doe', // This would come from your auth service
                email: 'user@example.com', // This would come from the verified token
                avatar: null,
                createdAt: new Date(),
                updatedAt: new Date(),
            };

            login(userData);
            return { success: true };
        } catch (error) {
            console.error('Magic link verification failed:', error);
            const errorMessage =
                error instanceof Error ? error.message : 'Invalid or expired magic link.';
            setError(errorMessage);
            return { success: false, error: errorMessage };
        } finally {
            setLoading(false);
        }
    },

    async loginWithGoogle(): Promise<AuthResult> {
        try {
            setLoading(true);
            clearError();

            // Your auth package Google login would go here
            // const result = await yourAuthPackage.loginWithGoogle();

            console.log('Logging in with Google');
            await new Promise((resolve) => setTimeout(resolve, 1000));

            const userData: User = {
                id: '456',
                name: 'John Doe',
                email: 'john@gmail.com',
                avatar: 'https://example.com/avatar.jpg',
                createdAt: new Date(),
                updatedAt: new Date(),
            };

            login(userData);
            return { success: true };
        } catch (error) {
            console.error('Google login failed:', error);
            const errorMessage =
                error instanceof Error ? error.message : 'Google login failed. Please try again.';
            setError(errorMessage);
            return { success: false, error: errorMessage };
        } finally {
            setLoading(false);
        }
    },

    async loginWithPasskey(): Promise<AuthResult> {
        try {
            setLoading(true);
            clearError();

            // Your auth package passkey login would go here
            // const result = await yourAuthPackage.loginWithPasskey();

            console.log('Logging in with passkey');
            await new Promise((resolve) => setTimeout(resolve, 1000));

            const userData: User = {
                id: '789',
                name: 'John Doe',
                email: 'john@example.com',
                isAdmin: true,
                avatar: null,
                createdAt: new Date(),
                updatedAt: new Date(),
            };

            login(userData);
            return { success: true };
        } catch (error) {
            console.error('Passkey login failed:', error);
            const errorMessage =
                error instanceof Error ? error.message : 'Passkey login failed. Please try again.';
            setError(errorMessage);
            return { success: false, error: errorMessage };
        } finally {
            setLoading(false);
        }
    },

    async logoutUser(): Promise<AuthResult> {
        try {
            setLoading(true);
            clearError();

            // Your auth package logout would go here
            // await yourAuthPackage.logout();

            console.log('Logging out');
            await new Promise((resolve) => setTimeout(resolve, 500));

            logout();
            return { success: true };
        } catch (error) {
            console.error('Logout failed:', error);
            const errorMessage =
                error instanceof Error ? error.message : 'Logout failed. Please try again.';
            setError(errorMessage);
            return { success: false, error: errorMessage };
        } finally {
            setLoading(false);
        }
    },

    async updateProfile(updates: Partial<User>): Promise<AuthResult> {
        try {
            setLoading(true);
            clearError();

            // Your auth package update profile would go here
            // const result = await yourAuthPackage.updateProfile(updates);

            console.log('Updating profile:', updates);
            await new Promise((resolve) => setTimeout(resolve, 500));

            updateUser({ ...updates, updatedAt: new Date() });
            return { success: true };
        } catch (error) {
            console.error('Profile update failed:', error);
            const errorMessage =
                error instanceof Error ? error.message : 'Failed to update profile.';
            setError(errorMessage);
            return { success: false, error: errorMessage };
        } finally {
            setLoading(false);
        }
    },
};
