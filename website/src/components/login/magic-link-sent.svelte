<!-- MagicLinkSent.svelte -->
<script lang="ts">
    import { authAPI } from '$lib/authState.svelte';

    interface Props {
        email?: string;
    }

    let { email }: Props = $props();

    async function handleDevLogin(): Promise<void> {
        await authAPI.verifyMagicLink('1234567890');
    }
</script>

<h1 class="title">Check your email for a magic link to continue!</h1>
<p class="subtitle">
    We've sent you an email{#if email}
        to <strong>{email}</strong>{/if} with a magic link to verify your account.
</p>

<!-- Development helper button -->
<button type="button" onclick={handleDevLogin} class="dev-button">
    enter app: (updates local storage state to enter app)
</button>

<style>
    .title {
        font-size: 1.5rem;
        font-weight: 600;
        margin: 0 0 1.5rem 0;
    }

    .subtitle {
        font-size: 0.875rem;
        color: #6b7280;
        margin-bottom: 2rem;
        line-height: 1.5;
    }

    .dev-button {
        background: #f3f4f6;
        border: 1px solid #d1d5db;
        color: #374151;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        font-size: 0.875rem;
        cursor: pointer;
        transition: background-color 0.2s;
    }

    .dev-button:hover {
        background: #e5e7eb;
    }
</style>
