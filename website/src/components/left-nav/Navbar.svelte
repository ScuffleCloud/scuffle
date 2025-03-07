<script lang="ts">
    import ScuffleLogo from '$lib/images/ScuffleLogo.svelte';
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
    <ul class="nav-links">
        <li><a href="/">Home</a></li>
        <li><a href="/about">About</a></li>
        <li><a href="/dashboard">Dashboard</a></li>
        <li><a href="/settings">Settings</a></li>
    </ul>
</nav>

<style>
    .sidebar {
        width: 240px;
        height: 100vh;
        background-color: var(--color-dark100);
        color: var(--color-light100);
        display: flex;
        flex-direction: column;
        box-shadow: 2px 0 5px rgba(0, 0, 0, 0.1);
        position: sticky;
        top: 0;

        .logo-container {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            color: var(--color-brown800);
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

        .nav-links {
            list-style: none;
            padding: 0;
            margin: 0;

            li {
                margin-bottom: 0.5rem;

                a {
                    display: block;
                    padding: 0.75rem 1rem;
                    color: var(--color-light100);
                    text-decoration: none;
                    border-radius: 4px;
                    transition: background-color 0.2s;

                    &:hover {
                        background-color: rgba(255, 255, 255, 0.1);
                    }
                }
            }
        }
    }
</style>
