<script lang="ts">
    import SettingsBlock from "$components/settings-block.svelte";
    import SettingsCard from "$components/settings-card.svelte";
    import Switch from "$components/switch.svelte";
    import { authState } from "$lib/auth.svelte";
    import { usersServiceClient } from "$lib/grpcClient";
    import IconBell from "$lib/images/icon-bell.svelte";
    import IconShield from "$lib/images/icon-shield.svelte";
    import { useTotpAuth } from "$lib/two-factor/toptAuth.svelte";
    import { useWebauthnAuth } from "$lib/two-factor/webAuthn.svelte";
    import type { UserSettings } from "$msw/mocks/settings";
    import { createQuery } from "@tanstack/svelte-query";
    import TwoFactorSettingsCard, {
        type MfaMethod,
    } from "./two-factor-settings-card.svelte";

    interface Props {
        settings: UserSettings;
    }

    const { settings }: Props = $props();

    const user = authState().user;
    let userSettings = $state(settings);

    const toptListQuery = createQuery(() => ({
        queryKey: ["totp-list"],
        queryFn: () =>
            usersServiceClient.listTotpCredentials({
                id: user!.id,
            }),
        enabled: !!user,
    }));

    const webauthnListQuery = createQuery(() => ({
        queryKey: ["webauthn-list"],
        queryFn: () =>
            usersServiceClient.listWebauthnCredentials({
                id: user!.id,
            }),
        enabled: !!user,
    }));

    const activeMethods = $state([
        {
            id: "1",
            name: "Google Authenticator",
            type: "TOTP",
            isPrimary: true,
        },
        { id: "2", name: "Passkey", type: "WEBAUTH" },
        { id: "3", name: "Passkey", type: "WEBAUTH" },
    ]);

    const webauthnAuth = useWebauthnAuth();

    const totpAuth = useTotpAuth();

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

    const hasAnyError = $derived(
        toptListQuery.isError || webauthnListQuery.isError,
    );

    const isLoading = $derived(
        toptListQuery.isLoading || webauthnListQuery.isLoading
            || hasAnyError,
    );

    let errorShown = $state(false);

    $effect(() => {
        if (hasAnyError && !errorShown) {
            errorShown = true;
        }
        if (toptListQuery.isSuccess && webauthnListQuery.isSuccess) {
            errorShown = false;
        }
    });

    // All cards should be stuck in loading until all queries pass
</script>

<div class="settings-page">
    <!-- WEBAUTHN -->
    <div class="two-factor-auth">
        Here:
        <!-- <button onclick={() => webAuthList()}>
            Load WebAuthn Credentials
        </button> -->
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
        <!-- <button onclick={() => totpList()}>
            Load TOTP Credentials
        </button> -->
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
        icon={IconShield}
    >
        <TwoFactorSettingsCard
            methods={activeMethods as MfaMethod[]}
            {isLoading}
        />
        <SettingsCard
            title="Recovery Codes"
            description="Generate backup codes to access your account if you lose your authenticator device."
            {isLoading}
        >
            <button
                class="action-button action-secondary"
                disabled={!userSettings.twoFactorAuth.enabled}
                onclick={() => {
                    userSettings.twoFactorAuth
                        .backupCodesGenerated = true;
                    console.log("Recovery codes generated");
                }}
            >
                {
                    userSettings.twoFactorAuth
                        .backupCodesGenerated
                    ? "Regenerate Codes"
                    : "Generate Codes"
                }
            </button>
        </SettingsCard>
    </SettingsBlock>

    <!-- Notification Settings -->
    <SettingsBlock
        title="Notification Settings"
        icon={IconBell}
    >
        <SettingsCard
            title="Email Notifications"
            description="Receive notifications about important account activities via email."
            {isLoading}
        >
            <Switch
                checked={userSettings.notifications.email.enabled}
                onchange={(checked) => {
                    userSettings.notifications.email.enabled =
                        checked;
                    console.log(
                        "Email notifications toggled:",
                        checked,
                    );
                }}
                showStateText={true}
                size="medium"
            />
        </SettingsCard>

        <SettingsCard
            title="Marketing Communications"
            description="Receive updates about new features and promotional offers."
        >
            <Switch
                checked={userSettings.notifications.email.marketing}
                onchange={(checked) => {
                    userSettings.notifications.email.marketing =
                        checked;
                    console.log(
                        "Marketing emails toggled:",
                        checked,
                    );
                }}
                showStateText={true}
                size="medium"
            />
        </SettingsCard>
    </SettingsBlock>

    <!-- Security & Privacy -->
    <SettingsBlock
        title="Security & Privacy"
        icon={IconShield}
    >
        <SettingsCard
            title="Critical Notifications"
            description="Receive critical notifications about important account activities."
        >
            <Switch
                checked={userSettings.notifications.email.criticalAlerts}
                onchange={(checked) => {
                    userSettings.notifications.email
                        .criticalAlerts = checked;
                    console.log(
                        "Critical notifications toggled:",
                        checked,
                    );
                }}
                showStateText={true}
                enabledText="On"
                disabledText="Off"
                size="medium"
            />
        </SettingsCard>

        <SettingsCard
            title="Password"
            description="Change your password to keep your account secure."
        >
            <button
                class="action-button action-secondary"
                onclick={() => console.log("Change password clicked")}
            >
                Change Password
            </button>
        </SettingsCard>
    </SettingsBlock>
</div>

<style>
    .settings-page {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }
</style>
