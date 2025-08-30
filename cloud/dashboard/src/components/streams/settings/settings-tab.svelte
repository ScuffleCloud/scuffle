<script lang="ts">
	import SettingsBlock from "$components/settings-block.svelte";
    import type { Card } from "$components/settings-block.svelte";
    import IconShield from "$lib/images/icon-shield.svelte";

    const VodCard = $derived<Card[]>([
        {
            id: "asset-management",
            title: "VOD Asset recording",
            description:
                "Manage your assets and their associated metadata.",
            actions: [
                {
                    variant: "toggle",
                    isToggled: true,
                    onClick: () => {
                        console.log("VOD Asset recording toggled");
                    },
                },
            ],
        },
    ]);
</script>

<div class="assets-page">
	<SettingsBlock title="Assets" cards={VodCard} icon={IconShield} />
</div>

<style>
</style>
