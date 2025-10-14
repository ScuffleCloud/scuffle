<script lang="ts">
    import SettingsBlock from "$lib/components/settings-block.svelte";
    import SettingsCard from "$lib/components/settings-card.svelte";
    import Switch from "$lib/components/switch.svelte";
    import IconShield from "$lib/images/icon-shield.svelte";

    let vodRecordingEnabled = $state(true);
</script>

<div class="assets-page">
    <SettingsBlock title="Assets" icon={IconShield}>
        <SettingsCard
            title="VOD Asset recording"
            description="Manage your assets and their associated metadata."
        >
            <Switch
                checked={vodRecordingEnabled}
                onchange={(checked) => {
                    vodRecordingEnabled = checked;
                    console.log("VOD Asset recording toggled");
                }}
                showStateText={true}
                size="medium"
            />
        </SettingsCard>
    </SettingsBlock>
</div>
