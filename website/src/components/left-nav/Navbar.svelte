<script lang="ts">
    import ScuffleLogo from '$lib/images/ScuffleLogo.svelte';
    import NavLinks from './nav-links.svelte';
    import OrganizationInfo from './OrganizationToggle.svelte';
</script>

<nav class="sidebar">
    <a class="logo-container" href="/">
        <div class="logo-container-image">
            <ScuffleLogo />
        </div>
        scuffle
    </a>
    <OrganizationInfo />
    <NavLinks />
</nav>

<style>
    .sidebar {
        width: 240px;
        height: 100vh;
        display: flex;
        flex-direction: column;
        position: sticky;
        top: 0;
        background-color: var(--color-teal30);
        .logo-container {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            font-size: 1.5rem;
            font-weight: 800;
            padding: 1rem;

            text-transform: uppercase;
            text-decoration: none;

            .logo-container-image {
                display: flex;
                align-items: center;
                justify-content: center;
                filter: drop-shadow(0px 2px 4px 0px rgb(0, 0, 0, 0.05));
            }
        }
    }
</style>
