<script lang="ts">
    import LoginOrDivider from "$lib/components/login-or-divider.svelte";
    import IconGoogle from "$lib/images/icon-google.svelte";
    import IconLoginKey from "$lib/images/icon-login-key.svelte";
    import type { LoginMode } from "./types";

    interface Props {
        onSubmit: () => Promise<void>;
        onModeChange: (mode: LoginMode) => void;
        isLoading?: boolean;
    }

    let { onSubmit, onModeChange, isLoading = false }: Props = $props();

    function handlePasskeyLogin() {
        onModeChange("passkey");
    }
</script>

<LoginOrDivider />
<button
    type="button"
    onclick={onSubmit}
    class="btn-social google"
    disabled={isLoading}
>
    <IconGoogle />
    Continue with Google
</button>

<button
    type="button"
    onclick={handlePasskeyLogin}
    class="btn-social passkey"
    disabled={isLoading}
>
    <IconLoginKey />
    Continue with Passkey
</button>

<style>
    .btn-social {
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

    .btn-social:hover:not(:disabled) {
      background: #f9fafb;
      border-color: #9ca3af;
    }

    .btn-social:disabled {
      background: white;
      color: #9ca3af;
      cursor: not-allowed;
    }
</style>
