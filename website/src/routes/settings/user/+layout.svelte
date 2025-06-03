<script lang="ts">
    import type { Snippet } from 'svelte';
    import Tabs from '$components/tabs.svelte';
    import IconPuzzle from '$lib/images/icon-puzzle.svelte';
    import IconGlobe from '$lib/images/icon-globe.svelte';
    import IconWallet from '$lib/images/icon-wallet.svelte';

    type Props = {
        children: Snippet;
    };

    const { children }: Props = $props();

    const tabs = [
        { id: 'common', label: 'Common', icon: IconPuzzle },
        { id: 'domains', label: 'Domains', icon: IconGlobe },
        { id: 'billing', label: 'Billing', icon: IconWallet },
    ];

    const baseUrl = '/settings/user/';
</script>

<div class="page-bg">
    <div class="header">
        <h1 class="page-title">User Settings</h1>
    </div>

    <Tabs {tabs} {baseUrl} defaultTab="common">
        {@render children()}
    </Tabs>
</div>

<style>
    .page-bg {
        background-color: var(--colors-light100);
        margin: 0 auto;
        width: 100%;
        max-width: 1200px;
        padding: 2rem;
    }

    .header {
        margin-bottom: 2rem;

        .page-title {
            font-size: 2rem;
            font-weight: 700;
            color: var(--color-yellow90);
            margin: 0;
        }
    }
</style>
