<script lang="ts">
    import { authState } from "$lib/auth.svelte";
    import SettingsBlock from "$lib/components/settings-block.svelte";
    import { queryKeys } from "$lib/consts";
    import {
        sessionsServiceClient,
        usersServiceClient,
    } from "$lib/grpcClient";
    import IconSettings2 from "$lib/images/icon-settings2.svelte";
    import IconShield from "$lib/images/icon-shield.svelte";
    import { createQuery } from "@tanstack/svelte-query";
    import {
        DEFAULT_TOTP_AUTH_NAME,
        DEFAULT_WEBAUTHN_AUTH_NAME,
    } from "./manage-two-factor/consts";
    import TwoFactorSettingsCard from "./manage-two-factor/two-factor-settings-card.svelte";
    import { type MfaCredential } from "./manage-two-factor/types";
    import PasswordSettingsCard from "./password/password-settings-card.svelte";
    import SessionsSettingsCard from "./sessions/sessions-settings-card.svelte";

    const userId = authState().user?.id;

    const totpListQuery = createQuery(() => ({
        queryKey: queryKeys.totp(userId!),
        queryFn: async () => {
            const call = usersServiceClient.listTotpCredentials({
                id: userId!,
            });
            const response = await call.response;
            return response.credentials;
        },
        enabled: !!userId,
    }));

    const webauthnListQuery = createQuery(() => ({
        queryKey: queryKeys.webauthn(userId!),
        queryFn: async () => {
            const call = usersServiceClient.listWebauthnCredentials({
                id: userId!,
            });
            const response = await call.response;
            return response.credentials;
        },
        enabled: !!userId,
    }));

    const sessionsQuery = createQuery(() => ({
        queryKey: queryKeys.sessions(userId!),
        queryFn: async () => {
            const call = sessionsServiceClient.list({
                id: userId!,
            });
            const response = await call.response;
            return response.sessions;
        },
        enabled: !!userId,
        initialData: [],
    }));

    const isLoading = $derived(
        totpListQuery.isLoading || webauthnListQuery.isLoading
            || totpListQuery.isError || webauthnListQuery.isError
            || sessionsQuery.isError || sessionsQuery.isLoading,
    );

    const authCredentials: MfaCredential[] = $derived(
        (() => {
            const totpCreds = (totpListQuery.data || []).map((
                cred,
            ) => ({
                ...cred,
                type: "totp" as const,
                name: cred.name || DEFAULT_TOTP_AUTH_NAME,
            }));

            const webauthnCreds = (webauthnListQuery.data || []).map((
                cred,
            ) => ({
                ...cred,
                type: "webauthn" as const,
                name: cred.name || DEFAULT_WEBAUTHN_AUTH_NAME,
            }));

            return [...totpCreds, ...webauthnCreds];
        })(),
    );
</script>

<div class="settings-page">
    <SettingsBlock
        title="Sign in methods"
        icon={IconShield}
    >
        <PasswordSettingsCard
            {isLoading}
            hasPassword={!!authState().user?.hasPassword}
        />
        <TwoFactorSettingsCard
            methods={authCredentials}
            {isLoading}
        />
    </SettingsBlock>
    <SettingsBlock
        title="Sessions"
        icon={IconSettings2}
    >
        <SessionsSettingsCard {isLoading} sessions={sessionsQuery.data || []} />
    </SettingsBlock>
</div>

<style>
    .settings-page {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }
</style>
