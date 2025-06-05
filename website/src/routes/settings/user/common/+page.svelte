<script lang="ts">
    import SettingsPage from '$components/settings/settings-page.svelte';
    import type { PageData } from './$types';

    interface Props {
        data: PageData;
    }

    const { data }: Props = $props();
</script>

<svelte:head>
    <title>Account Settings</title>
    <meta name="description" content="Manage your account settings, security, and preferences" />
</svelte:head>

{#await data.settings}
    <div class="loading-state">
        <p>Loading your settings. TODO: Add some skeleton loading...</p>
    </div>
{:then settings}
    <SettingsPage {settings} />
{:catch error}
    <h2>Failed to load settings</h2>
{/await}

<style>
    .loading-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        min-height: 50vh;
    }
</style>
