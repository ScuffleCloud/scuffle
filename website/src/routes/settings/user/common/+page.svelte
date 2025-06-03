<!-- Example usage of SettingsBlock for Two-factor Authentication -->
<script lang="ts">
    import SettingsBlock from '$components/settings-block.svelte';
    import type { Card } from '$components/settings-block.svelte';
    import IconShield from '$lib/images/icon-shield.svelte';
    import IconBell from '$lib/images/icon-bell.svelte';

    let is2FAEnabled = false;
    let notificationsEnabled = true;

    const cards: Card[] = [
        {
            id: 'two-factor-auth',
            title: 'Two-factor authentication',
            description:
                'Enables a second layer of security, by requiring at least two methods of authentication for signing in.',
            status: {
                label: is2FAEnabled ? 'Enabled' : 'Disabled',
                variant: is2FAEnabled ? 'enabled' : 'disabled',
            },
            actions: [
                {
                    label: 'Enable 2FA',
                    variant: 'primary',
                    onClick: () => {
                        is2FAEnabled = !is2FAEnabled;
                        console.log('2FA toggled:', is2FAEnabled);
                    },
                },
            ],
        },
        {
            id: 'recovery-codes',
            title: 'Recovery Codes',
            subtitle: 'Subtext about this temporary text',
            actions: [
                {
                    label: 'Regenerate Codes',
                    variant: 'secondary',
                    onClick: () => {
                        console.log('Regenerating recovery codes');
                    },
                },
            ],
        },
    ];
</script>

<div class="settings-block-wrapper">
    <SettingsBlock title="Two-factor Authentication" subtitle="(2FA)" {cards} icon={IconShield} />

    <SettingsBlock
        icon={IconShield}
        title="Notification Settings"
        cards={[
            {
                id: 'email-notifications',
                title: 'Email Notifications',
                description: 'Receive notifications about important account activities via email.',
                actions: [
                    {
                        variant: 'toggle',
                        isToggled: notificationsEnabled,
                        onClick: () => {
                            notificationsEnabled = !notificationsEnabled;
                            console.log('Notifications toggled:', notificationsEnabled);
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
                        isToggled: false,
                        onClick: () => console.log('Marketing toggle'),
                    },
                    {
                        label: 'Manage Preferences',
                        variant: 'secondary',
                        onClick: () => console.log('Manage preferences'),
                    },
                ],
            },
        ]}
    />

    <SettingsBlock
        icon={IconBell}
        title="Notifications"
        cards={[
            {
                id: 'critical-notifications',
                title: 'Critical Notifications',
                description: 'Receive critical notifications about important account activities.',
                status: {
                    label: 'Enabled',
                    variant: 'enabled',
                },
                actions: [
                    {
                        variant: 'toggle',
                        isToggled: true,
                        onClick: () => console.log('Edit profile'),
                    },
                ],
            },
            {
                id: 'password',
                title: 'Password',
                description: 'Change your password to keep your account secure.',
                status: {
                    label: 'Strong',
                    variant: 'enabled',
                },
                actions: [
                    {
                        label: 'Change Password',
                        variant: 'secondary',
                        onClick: () => console.log('Change password'),
                    },
                ],
            },
        ]}
    />

    <SettingsBlock
        icon={IconShield}
        title="Account Settings"
        cards={[
            {
                id: 'profile',
                title: 'Profile Information',
                description: 'Update your name, email, and other profile details.',
                actions: [
                    {
                        label: 'Edit Profile',
                        variant: 'primary',
                        onClick: () => console.log('Edit profile'),
                    },
                ],
            },
            {
                id: 'password',
                title: 'Password',
                description: 'Change your password to keep your account secure.',
                status: {
                    label: 'Strong',
                    variant: 'enabled',
                },
                actions: [
                    {
                        label: 'Change Password',
                        variant: 'secondary',
                        onClick: () => console.log('Change password'),
                    },
                ],
            },
        ]}
    />

    <SettingsBlock
        icon={IconShield}
        title="Danger Zone"
        cards={[
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
                        onClick: () => console.log('Delete account'),
                    },
                ],
            },
        ]}
    />
</div>

<style>
    .settings-block-wrapper {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }
</style>
