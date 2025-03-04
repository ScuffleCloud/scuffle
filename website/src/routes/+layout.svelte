<script lang="ts">
    import { theme } from '$lib/theme';
    import '../styles/global.css';

    const { children } = $props();

    const cssVariables = Object.entries(theme.colors)
        .map(([key, value]) => `--color-${key}: ${value}`)
        .join(';');
</script>

<div class="app" style={cssVariables}>
    <main>
        {@render children()}
    </main>
</div>

<style>
    .app {
        position: relative;
        display: flex;
        flex-direction: column;
        min-height: 100vh;
        background-color: var(--color-light100);
    }

    main {
        position: relative;
        flex: 1;
        display: flex;
        flex-direction: column;
        box-sizing: border-box;
    }
</style>
