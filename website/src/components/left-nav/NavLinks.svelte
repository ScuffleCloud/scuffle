<script>
    import { NAV_ITEMS } from './consts';
</script>

<ul class="nav-links">
    {#each NAV_ITEMS as item}
        <li><a href={item.path}>{item.label}</a></li>
    {/each}
    <li><a href="/about">About</a></li>
    <li><a href="/dashboard">Dashboard</a></li>
    <li><a href="/settings">Settings</a></li>
</ul>

<style>
    .nav-links {
        list-style: none;
        padding: 0;
        margin: 0;

        li {
            margin-bottom: 0.5rem;

            a {
                display: block;
                padding: 0.75rem 1rem;
                text-decoration: none;
                border-radius: 4px;
                transition: background-color 0.2s;

                &:hover {
                    background-color: rgba(255, 255, 255, 0.1);
                }
            }
        }
    }
</style>
