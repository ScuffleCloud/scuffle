<script lang="ts">
    import IconEyeClosed from "$lib/images/icon-eye_closed.svelte";
    import IconEyeOpen from "$lib/images/icon-eye_open.svelte";

    interface Props {
        id: string;
        label: string;
        value: string;
        placeholder?: string;
        disabled?: boolean;
        onchange?: (value: string) => void;
    }

    let {
        id,
        label,
        value = $bindable(),
        placeholder = "",
        disabled = false,
    }: Props = $props();

    let showPassword = $state(false);
</script>

<div class="input-group">
    <label for={id}>{label}</label>
    <div class="password-input-wrapper">
        <input
            {id}
            type={showPassword ? "text" : "password"}
            bind:value
            {placeholder}
            {disabled}
            class="password-input"
        />
        <button
            type="button"
            class="toggle-password"
            onclick={() => (showPassword = !showPassword)}
            aria-label={showPassword ? "Hide password" : "Show password"}
        >
            {#if showPassword}
                <IconEyeClosed />
            {:else}
                <IconEyeOpen />
            {/if}
        </button>
    </div>
</div>

<style>
    .input-group {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }

    .input-group label {
      font-size: 0.875rem;
      font-weight: 500;
      color: #374151;
    }

    .password-input-wrapper {
      position: relative;
      display: flex;
      align-items: center;
    }

    .password-input {
      width: 100%;
      padding: 1rem;
      padding-right: 3rem;
      border: 1px solid #e5e5e5;
      border-radius: 0.75rem;
      font-size: 1rem;
      background-color: #f9fafb;
    }

    .password-input:focus {
      outline: none;
      border-color: rgb(247, 177, 85);
      background-color: white;
    }

    .password-input:disabled {
      cursor: not-allowed;
      opacity: 0.6;
    }

    .toggle-password {
      position: absolute;
      right: 1rem;
      background: none;
      border: none;
      cursor: pointer;
      padding: 0.5rem;
      display: flex;
      align-items: center;
      justify-content: center;
      color: #71717a;
    }

    .toggle-password:hover {
      color: #18181b;
    }
</style>
