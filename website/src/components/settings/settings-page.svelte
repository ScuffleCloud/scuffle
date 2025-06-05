<script lang="ts">
    import SettingsBlock from '$components/settings-block.svelte';
    import IconShield from '$lib/images/icon-shield.svelte';
    import IconBell from '$lib/images/icon-bell.svelte';
    import type { UserSettings } from '$msw/mocks/settings';
    import type { Card } from '$components/settings-block.svelte';

    interface Props {
        settings: UserSettings;
    }

    const { settings }: Props = $props();

    let userSettings = $state(settings);

    const twoFactorCards = $derived<Card[]>([
        {
            id: 'two-factor-auth',
            title: 'Two-factor authentication',
            description:
                'Enables a second layer of security, by requiring at least two methods of authentication for signing in.',
            status: {
                label: userSettings.twoFactorAuth.enabled ? 'Enabled' : 'Disabled',
                variant: userSettings.twoFactorAuth.enabled ? 'enabled' : 'disabled',
            },
            actions: [
                {
                    label: userSettings.twoFactorAuth.enabled ? 'Disable 2FA' : 'Enable 2FA',
                    variant: 'primary',
                    onClick: () => {
                        userSettings.twoFactorAuth.enabled = !userSettings.twoFactorAuth.enabled;
                        console.log('2FA toggled:', userSettings.twoFactorAuth.enabled);
                    },
                },
            ],
        },
        {
            id: 'recovery-codes',
            title: 'Recovery Codes',
            description:
                'Generate backup codes to access your account if you lose your authenticator device.',
            actions: [
                {
                    label: userSettings.twoFactorAuth.backupCodesGenerated
                        ? 'Regenerate Codes'
                        : 'Generate Codes',
                    variant: 'secondary',
                    disabled: !userSettings.twoFactorAuth.enabled,
                    onClick: () => {
                        userSettings.twoFactorAuth.backupCodesGenerated = true;
                        console.log('Recovery codes generated');
                    },
                },
            ],
        },
    ]);

    // Notification Cards
    const notificationCards = $derived<Card[]>([
        {
            id: 'email-notifications',
            title: 'Email Notifications',
            description: 'Receive notifications about important account activities via email.',
            actions: [
                {
                    variant: 'toggle',
                    isToggled: userSettings.notifications.email.enabled,
                    onClick: () => {
                        userSettings.notifications.email.enabled =
                            !userSettings.notifications.email.enabled;
                        console.log(
                            'Email notifications toggled:',
                            userSettings.notifications.email.enabled,
                        );
                    },
                },
            ],
        },
        {
            id: 'marketing-emails',
            title: 'Marketing Communications',
            description: 'Receive updates about new features and promotional offers.',
            actions: [
                {
                    variant: 'toggle',
                    isToggled: userSettings.notifications.email.marketing,
                    onClick: () => {
                        userSettings.notifications.email.marketing =
                            !userSettings.notifications.email.marketing;
                        console.log(
                            'Marketing emails toggled:',
                            userSettings.notifications.email.marketing,
                        );
                    },
                },
            ],
        },
    ]);

    // Security & Privacy Cards
    const securityCards = $derived<Card[]>([
        {
            id: 'critical-notifications',
            title: 'Critical Notifications',
            description: 'Receive critical notifications about important account activities.',
            actions: [
                {
                    variant: 'toggle',
                    isToggled: userSettings.notifications.email.criticalAlerts,
                    enabledText: 'On',
                    disabledText: 'Off',
                    onClick: () => {
                        userSettings.notifications.email.criticalAlerts =
                            !userSettings.notifications.email.criticalAlerts;
                        console.log(
                            'Critical notifications toggled:',
                            userSettings.notifications.email.criticalAlerts,
                        );
                    },
                },
            ],
        },
        {
            id: 'password',
            title: 'Password',
            description: 'Change your password to keep your account secure.',
            actions: [
                {
                    label: 'Change Password',
                    variant: 'secondary',
                    onClick: () => {
                        console.log('Change password clicked');
                    },
                },
            ],
        },
    ]);

    // Account Settings Cards
    const accountCards = $derived<Card[]>([
        {
            id: 'profile',
            title: 'Profile Information',
            description: 'Update your name, email, and other profile details.',
            actions: [
                {
                    label: 'Edit Profile',
                    variant: 'primary',
                    onClick: () => {
                        console.log('Edit profile clicked');
                    },
                },
            ],
        },
        {
            id: 'preferences',
            title: 'Display Preferences',
            description: 'Customize your account display settings and preferences.',
            actions: [
                {
                    variant: 'toggle',
                    isToggled: userSettings.preferences.darkMode,
                    enabledText: 'Dark Mode',
                    disabledText: 'Light Mode',
                    onClick: () => {
                        userSettings.preferences.darkMode = !userSettings.preferences.darkMode;
                        console.log('Dark mode toggled:', userSettings.preferences.darkMode);
                    },
                },
            ],
        },
        {
            id: 'auto-save',
            title: 'Auto-save Settings',
            description: 'Automatically save your work as you make changes.',
            actions: [
                {
                    variant: 'toggle',
                    isToggled: userSettings.preferences.autoSave,
                    enabledText: 'Enabled',
                    disabledText: 'Disabled',
                    onClick: () => {
                        userSettings.preferences.autoSave = !userSettings.preferences.autoSave;
                        console.log('Auto-save toggled:', userSettings.preferences.autoSave);
                    },
                },
            ],
        },
    ]);

    // Danger Zone Cards
    const dangerCards = $derived<Card[]>([
        {
            id: 'delete-account',
            title: 'Delete Account',
            description:
                'Permanently delete your account and all associated data. This action cannot be undone.',
            status: {
                label: 'Irreversible',
                variant: 'warning',
            },
            actions: [
                {
                    label: 'Delete Account',
                    variant: 'danger',
                    onClick: () => {
                        console.log('Delete account clicked');
                        // In a real app, you'd show a confirmation modal here
                    },
                },
            ],
        },
    ]);
</script>

<div class="settings-page">
    <SettingsBlock
        title="Two-factor Authentication"
        subtitle="(2FA)"
        cards={twoFactorCards}
        icon={IconShield}
    />

    <SettingsBlock title="Notification Settings" cards={notificationCards} icon={IconBell} />

    <SettingsBlock title="Security & Privacy" cards={securityCards} icon={IconShield} />

    <!-- <SettingsBlock title="Account Settings" cards={accountCards} icon={IconUser} />

        <SettingsBlock title="Danger Zone" cards={dangerCards} icon={IconAlertTriangle} /> -->
</div>

<style>
    .settings-page {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }
</style>
