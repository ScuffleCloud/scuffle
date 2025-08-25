<script lang="ts">
    const { organizations } = $props();
</script>

<div class="org-content">
    <h3 class="dropdown-title">Switch organization</h3>
    {#if organizations}
        <ul class="org-list">
            {#each organizations as org}
                <li class="org-item">
                    <button class="org-button">
                        <div class="mini-avatar" style:background-color="#FFCC80"></div>
                        <span class="org-name-dropdown">{org.name}</span>
                    </button>
                </li>
            {/each}
        </ul>
    {/if}
</div>

<style>
    .org-content {
        background-color: white;
        border-radius: 0.5rem;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        padding: 0.75rem;
        margin-top: 0.5rem;
        width: 100%;
        max-width: 250px;
        border: 1px solid rgba(0, 0, 0, 0.1);

        .dropdown-title {
            font-size: 0.75rem;
            color: var(--colors-brown-600);
            margin-bottom: 0.5rem;
            font-weight: 600;
            padding-left: 0.25rem;
        }

        .org-list {
            list-style: none;
            padding: 0;
            margin: 0;

            .org-item {
                .org-button {
                    display: flex;
                    align-items: center;
                    width: 100%;
                    padding: 0.5rem;
                    border: none;
                    background: none;
                    text-align: left;
                    cursor: pointer;
                    border-radius: 0.25rem;
                    font-family: inherit;

                    &:hover {
                        background-color: rgba(0, 0, 0, 0.05);
                    }

                    .mini-avatar {
                        width: 1.5rem;
                        height: 1.5rem;
                        border-radius: 0.25rem;
                        margin-right: 0.5rem;
                        flex-shrink: 0;
                    }

                    .org-name-dropdown {
                        font-size: 0.875rem;
                        font-weight: 500;
                        color: var(--colors-brown-600);
                    }
                }
            }
        }
    }
</style>
