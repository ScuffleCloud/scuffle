export interface UserSettings {
    id: string;
    userId: string;
    twoFactorAuth: {
        enabled: boolean;
        method: 'app' | 'sms' | null;
        backupCodesGenerated: boolean;
        enabledAt: string | null;
    };
    notifications: {
        email: {
            enabled: boolean;
            security: boolean;
            marketing: boolean;
            criticalAlerts: boolean;
        };
        push: {
            enabled: boolean;
            criticalAlerts: boolean;
        };
    };
    security: {
        passwordStrength: 'weak' | 'medium' | 'strong';
        lastPasswordChange: string;
        sessionTimeout: number;
    };
    profile: {
        displayName: string;
        bio: string | null;
        timezone: string;
        language: string;
    };
    preferences: {
        darkMode: boolean;
        compactView: boolean;
        autoSave: boolean;
    };
    createdAt: string;
    updatedAt: string;
}

export const mockUserSettingsResponse: UserSettings = {
    id: 'settings_123',
    userId: 'usr_123',
    twoFactorAuth: {
        enabled: false,
        method: null,
        backupCodesGenerated: false,
        enabledAt: null,
    },
    notifications: {
        email: {
            enabled: true,
            security: true,
            marketing: false,
            criticalAlerts: true,
        },
        push: {
            enabled: true,
            criticalAlerts: true,
        },
    },
    security: {
        passwordStrength: 'strong',
        lastPasswordChange: '2024-03-15T10:30:00Z',
        sessionTimeout: 3600,
    },
    profile: {
        displayName: 'John Doe',
        bio: 'Software Developer at Acme Corp',
        timezone: 'America/New_York',
        language: 'en-US',
    },
    preferences: {
        darkMode: true,
        compactView: false,
        autoSave: true,
    },
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-06-05T14:30:00Z',
};
