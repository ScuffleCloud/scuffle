<script lang="ts">
    type Status = 'live' | 'finished';
    export let status: Status;
</script>

<span class="status-badge" class:live={status === 'live'} class:finished={status === 'finished'}>
    {#if status === 'live'}
        <svg width="6" height="6" viewBox="0 0 6 6" fill="none" xmlns="http://www.w3.org/2000/svg">
            <rect width="6" height="6" rx="3" fill="#A7133A" />
        </svg>
    {/if}
    {status}
</span>

<style>
    .status-badge {
        display: flex;
        align-items: center;
        gap: 0.25rem;
        padding: 0.25rem 0.5rem;
        border-radius: 0.25rem;
        font-size: 0.875rem;
        font-weight: 500;
        text-transform: capitalize;

        &.live {
            padding: 4px 9px;
            color: var(--colors-red90);
            background-color: var(--colors-red30);
            border-radius: 100rem;
        }

        &.finished {
            color: #666;
            background-color: #f0f0f0;
        }
    }
</style>
