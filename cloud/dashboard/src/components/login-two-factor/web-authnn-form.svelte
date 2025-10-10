<script lang="ts">
    import LoginOrDivider from "$components/login-or-divider.svelte";
    import IconShield from "$lib/images/icon-shield.svelte";
    import { onMount } from "svelte";
    import RecoveryCodeCollapsible from "./recovery-code-collapsible.svelte";

    interface Props {
        onToptModeChange: (() => void) | null;
        onBackupCodeChange: () => void;
    }

    let { onToptModeChange, onBackupCodeChange }: Props = $props();

    async function handleWebauthnChallenge() {
        console.log("handleWebauthnChallenge");
    }
</script>

<div class="header">
    <h1 class="title">Authentication</h1>
</div>
<p class="subtitle">
    Plug in your security key and touch it when prompted to continue.
</p>
<button
    type="button"
    onclick={handleWebauthnChallenge}
    class="continue-btn"
>
    Resend request
</button>
{#if onToptModeChange}
    <LoginOrDivider />
    <button
        type="button"
        onclick={() => onToptModeChange()}
        class="continue-btn"
    >
        <IconShield />
        Continue with 2FA code
    </button>
{/if}

<div class="separator"></div>
<RecoveryCodeCollapsible onAction={onBackupCodeChange} />

<style>
    .header {
      display: flex;
      align-items: center;
      position: relative;
      margin-bottom: 1rem;
    }

    .title {
      flex: 1;
      font-size: 1.5rem;
      font-weight: 600;
      color: #1f2937;
      margin: 0;
    }

    .subtitle {
      color: #6b7280;
      font-size: 0.95rem;
      line-height: 1.5;
      margin: 0 0 1rem 0;
    }

    .continue-btn {
      width: 100%;
      padding: 0.75rem;
      background: white;
      color: #374151;
      border: 1px solid #d1d5db;
      cursor: pointer;
      margin-bottom: 0.5rem;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
      border-radius: 0.5rem;
    }

    .continue-btn:hover:not(:disabled) {
      background: #f9fafb;
      border-color: #9ca3af;
    }

    .continue-btn:disabled {
      background: white;
      color: #9ca3af;
      cursor: not-allowed;
    }

    .separator {
      width: 100%;
      height: 1px;
      background: #e5e7eb;
      margin: 1.5rem 0 1rem 0;
    }
</style>
