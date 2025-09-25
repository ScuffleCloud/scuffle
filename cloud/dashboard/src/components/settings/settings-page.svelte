<script lang="ts">
    import SettingsBlock from "$components/settings-block.svelte";
    import type { Card } from "$components/settings-block.svelte";
    import { authState } from "$lib/auth.svelte";
    import { usersServiceClient } from "$lib/grpcClient";
    import IconBell from "$lib/images/icon-bell.svelte";
    import IconShield from "$lib/images/icon-shield.svelte";
    import { useTotpAuth } from "$lib/two-factor/toptAuth.svelte";
    import { useWebauthnAuth } from "$lib/two-factor/webAuthn.svelte";
    import type { UserSettings } from "$msw/mocks/settings";

    interface Props {
        settings: UserSettings;
    }

    const { settings }: Props = $props();

    const user = authState().user;
    let userSettings = $state(settings);

    // Load in user information on mfa stuff
    async function webAuthList() {
        if (!user) return;

        const listCredentials = usersServiceClient
            .listWebauthnCredentials({
                id: user.id,
            });
        const response = await listCredentials.response;
        console.log("response", response);
    }

    async function totpList() {
        if (!user) return;
        const listCredentials = usersServiceClient
            .listTotpCredentials({
                id: user.id,
            });
        const response = await listCredentials.response;
        console.log("response", response);
    }

    // Do the 2fa locally without hardening code structures because I don't know where it will go
    // Lets generate some UUID here for the name of the key. No reason to store it in the database
    // because we can just generate it on the fly.

    const webauthnAuth = useWebauthnAuth();

    const totpAuth = useTotpAuth();

    const twoFactorCards = $derived<Card[]>([
        {
            id: "two-factor-auth",
            title: "Two-factor authentication",
            description:
                "Enables a second layer of security, by requiring at least two methods of authentication for signing in.",
            status: {
                label: userSettings.twoFactorAuth.enabled
                    ? "Enabled"
                    : "Disabled",
                variant: userSettings.twoFactorAuth.enabled
                    ? "enabled"
                    : "disabled",
            },
            actions: [
                {
                    label: userSettings.twoFactorAuth.enabled
                        ? "Disable 2FA"
                        : "Enable 2FA",
                    variant: "primary",
                    onClick: () => {
                        userSettings.twoFactorAuth.enabled =
                            !userSettings
                                .twoFactorAuth
                                .enabled;
                        console.log(
                            "2FA toggled:",
                            userSettings.twoFactorAuth.enabled,
                        );
                    },
                },
            ],
        },
        {
            id: "recovery-codes",
            title: "Recovery Codes",
            description:
                "Generate backup codes to access your account if you lose your authenticator device.",
            actions: [
                {
                    label:
                        userSettings.twoFactorAuth.backupCodesGenerated
                            ? "Regenerate Codes"
                            : "Generate Codes",
                    variant: "secondary",
                    disabled: !userSettings.twoFactorAuth.enabled,
                    onClick: () => {
                        userSettings.twoFactorAuth
                            .backupCodesGenerated = true;
                        console.log("Recovery codes generated");
                    },
                },
            ],
        },
    ]);

    // Notification Cards
    const notificationCards = $derived<Card[]>([
        {
            id: "email-notifications",
            title: "Email Notifications",
            description:
                "Receive notifications about important account activities via email.",
            actions: [
                {
                    variant: "toggle",
                    isToggled: userSettings.notifications.email.enabled,
                    onClick: () => {
                        userSettings.notifications.email.enabled =
                            !userSettings
                                .notifications
                                .email.enabled;
                        console.log(
                            "Email notifications toggled:",
                            userSettings.notifications.email.enabled,
                        );
                    },
                },
            ],
        },
        {
            id: "marketing-emails",
            title: "Marketing Communications",
            description:
                "Receive updates about new features and promotional offers.",
            actions: [
                {
                    variant: "toggle",
                    isToggled:
                        userSettings.notifications.email.marketing,
                    onClick: () => {
                        userSettings.notifications.email.marketing =
                            !userSettings
                                .notifications.email.marketing;
                        console.log(
                            "Marketing emails toggled:",
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
            id: "critical-notifications",
            title: "Critical Notifications",
            description:
                "Receive critical notifications about important account activities.",
            actions: [
                {
                    variant: "toggle",
                    isToggled:
                        userSettings.notifications.email.criticalAlerts,
                    enabledText: "On",
                    disabledText: "Off",
                    onClick: () => {
                        userSettings.notifications.email
                            .criticalAlerts = !userSettings
                                .notifications.email.criticalAlerts;
                        console.log(
                            "Critical notifications toggled:",
                            userSettings.notifications.email
                                .criticalAlerts,
                        );
                    },
                },
            ],
        },
        {
            id: "password",
            title: "Password",
            description:
                "Change your password to keep your account secure.",
            actions: [
                {
                    label: "Change Password",
                    variant: "secondary",
                    onClick: () => {
                        console.log("Change password clicked");
                    },
                },
            ],
        },
    ]);

    // Account Settings Cards
    // const accountCards = $derived<Card[]>([
    //     {
    //         id: "profile",
    //         title: "Profile Information",
    //         description:
    //             "Update your name, email, and other profile details.",
    //         actions: [
    //             {
    //                 label: "Edit Profile",
    //                 variant: "primary",
    //                 onClick: () => {
    //                     console.log("Edit profile clicked");
    //                 },
    //             },
    //         ],
    //     },
    //     {
    //         id: "preferences",
    //         title: "Display Preferences",
    //         description:
    //             "Customize your account display settings and preferences.",
    //         actions: [
    //             {
    //                 variant: "toggle",
    //                 isToggled: userSettings.preferences.darkMode,
    //                 enabledText: "Dark Mode",
    //                 disabledText: "Light Mode",
    //                 onClick: () => {
    //                     userSettings.preferences.darkMode =
    //                         !userSettings
    //                             .preferences
    //                             .darkMode;
    //                     console.log(
    //                         "Dark mode toggled:",
    //                         userSettings.preferences.darkMode,
    //                     );
    //                 },
    //             },
    //         ],
    //     },
    //     {
    //         id: "auto-save",
    //         title: "Auto-save Settings",
    //         description:
    //             "Automatically save your work as you make changes.",
    //         actions: [
    //             {
    //                 variant: "toggle",
    //                 isToggled: userSettings.preferences.autoSave,
    //                 enabledText: "Enabled",
    //                 disabledText: "Disabled",
    //                 onClick: () => {
    //                     userSettings.preferences.autoSave =
    //                         !userSettings
    //                             .preferences
    //                             .autoSave;
    //                     console.log(
    //                         "Auto-save toggled:",
    //                         userSettings.preferences.autoSave,
    //                     );
    //                 },
    //             },
    //         ],
    //     },
    // ]);

    // Danger Zone Cards
    // const dangerCards = $derived<Card[]>([
    //     {
    //         id: "delete-account",
    //         title: "Delete Account",
    //         description:
    //             "Permanently delete your account and all associated data. This action cannot be undone.",
    //         status: {
    //             label: "Irreversible",
    //             variant: "warning",
    //         },
    //         actions: [
    //             {
    //                 label: "Delete Account",
    //                 variant: "danger",
    //                 onClick: () => {
    //                     console.log("Delete account clicked");
    //                 },
    //             },
    //         ],
    //     },
    // ]);

    // For webauthn
    let webauthnCredentialName = $state("");

    let totpCredentialName = $state("");
    let totpCode = $state("");
</script>

<div class="settings-page">
    <!-- WEBAUTHN -->
    <div class="two-factor-auth">
        Here:
        <button onclick={() => webAuthList()}>
            Load WebAuthn Credentials
        </button>
        <br>
        2fa information here testing flows:
        <br>
        <input type="text" bind:value={webauthnCredentialName} />
        <br>
        <button
            onclick={() =>
            webauthnAuth.createCredential(
                webauthnCredentialName,
            )}
        >
            Test WebAuthn
        </button>
        <br>
        Is supported: {webauthnAuth.isSupported()}
        <br>
        Loading: {webauthnAuth.loading()}
        <br>
        Error: {webauthnAuth.error()}
    </div>
    <!-- TOPT -->
    <div class="two-factor-auth">
        <button onclick={() => totpList()}>
            Load TOTP Credentials
        </button>
        <br>
        Here:
        <button onclick={() => totpAuth.initiateTotpSetup()}>
            Initiate TOTP Setup
        </button>
        {#if totpAuth.qrCodeData()}
            <div>
                QR Code generated! Scan with your authenticator app.
                <br>
                Secret URL: {totpAuth.qrCodeData()?.secretUrl}
            </div>
        {/if}

        <!-- Step 2: Complete setup after scanning QR -->
        <input
            type="text"
            bind:value={totpCredentialName}
            placeholder="Credential name"
        />
        <br>
        <input
            type="text"
            bind:value={totpCode}
            placeholder="6-digit code from app"
        />
        <br>
        <button
            onclick={() =>
            totpAuth.completeTotpSetup(
                totpCredentialName,
                totpCode,
            )}
            disabled={!totpAuth.qrCodeData() || !totpCredentialName
            || !totpCode}
        >
            Complete TOTP Setup
        </button>
        <br>

        Loading: {totpAuth.loading()}
        <br>
        Error: {totpAuth.error()}
        <br>
    </div>
    <SettingsBlock
        title="Two-factor Authentication"
        subtitle="(2FA)"
        cards={twoFactorCards}
        icon={IconShield}
    />

    <SettingsBlock
        title="Notification Settings"
        cards={notificationCards}
        icon={IconBell}
    />

    <SettingsBlock
        title="Security & Privacy"
        cards={securityCards}
        icon={IconShield}
    />
</div>

<style>
    .settings-page {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }
</style>
