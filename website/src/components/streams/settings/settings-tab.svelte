<script lang="ts">
    import SettingsBlock from '$components/settings-block.svelte';
    import type { Card } from '$components/settings-block.svelte';
    import IconShield from '$lib/images/icon-shield.svelte';

    // Keep track of the settings that are toggled
    let vodCardToggled = $state(true);
    let accessManagementCardToggled = $state(true);
    let accessManagementCardToggled2 = $state(true);
    let accessManagementCardToggled3 = $state(true);

    const VodCard = $derived<Card[]>([
        {
            id: 'asset-management',
            title: 'VOD Asset recording',
            description: 'Manage your assets and their associated metadata.',
            actions: [
                {
                    variant: 'toggle',
                    isToggled: vodCardToggled,
                    onClick: () => {
                        vodCardToggled = !vodCardToggled;
                    },
                },
            ],
        },
    ]);

    const AccessManagementCard = $derived<Card[]>([
        {
            id: 'asset-management',
            title: 'Access Management',
            description: 'Manage your access to the stream.',
            actions: [
                {
                    variant: 'toggle',
                    isToggled: accessManagementCardToggled,
                    onClick: () => {
                        accessManagementCardToggled = !accessManagementCardToggled;
                    },
                },
            ],
        },
        {
            id: 'access-management-2',
            title: 'Require Signed URLs',
            description: 'Manage your access to the stream.',
            actions: [
                {
                    variant: 'toggle',
                    isToggled: accessManagementCardToggled2,
                    onClick: () => {
                        accessManagementCardToggled2 = !accessManagementCardToggled2;
                    },
                },
            ],
        },
        {
            id: 'access-management-3',
            title: 'Require Signed URLs',
            description: 'Manage your access to the stream.',
            actions: [
                {
                    variant: 'toggle',
                    isToggled: accessManagementCardToggled3,
                    onClick: () => {
                        accessManagementCardToggled3 = !accessManagementCardToggled3;
                    },
                },
            ],
        },
    ]);
</script>

<div class="assets-page">
    <SettingsBlock title="Assets" cards={VodCard} icon={IconShield} />
    <SettingsBlock title="Access Management" cards={AccessManagementCard} icon={IconShield} />
</div>

<style>
    .assets-page {
        display: flex;
        flex-direction: column;
        gap: 2.25rem;
    }
</style>
