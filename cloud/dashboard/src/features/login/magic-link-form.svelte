<script lang="ts">
    import LoginFormTitle from "./login-form-title.svelte";

    interface Props {
        onSubmit: (email: string) => Promise<void>;
        isLoading: boolean;
    }

    let { onSubmit, isLoading }: Props = $props();

    async function handleSubmit(event: SubmitEvent): Promise<void> {
        event.preventDefault();
        const formData = new FormData(event.target as HTMLFormElement);
        const email = formData.get("email") as string;
        await onSubmit(email);
    }
</script>

<LoginFormTitle title="Log in to Scuffle" />
<form onsubmit={handleSubmit} class="login-form">
    <div class="form-group">
        <label for="email" class="form-label">Email</label>
        <input
            type="email"
            name="email"
            id="email"
            class="form-input"
            placeholder="Enter your email"
            disabled={isLoading}
            required
            autocomplete="email"
        />
    </div>

    <button type="submit" class="btn-primary" disabled={isLoading}>
        {#if isLoading}
            <span class="loading-spinner-small"></span>
            Logging in...
        {:else}
            Continue with email
        {/if}
    </button>
</form>

<style>
    .login-form {
      margin-bottom: 0.5rem;
    }

    .form-group {
      margin-bottom: 1.25rem;
      text-align: left;
    }

    .form-label {
      display: block;
      font-size: 0.875rem;
      font-weight: 500;
      color: #374151;
      margin-bottom: 0.375rem;
    }

    .form-input {
      width: 100%;
      padding: 0.75rem 1rem;
      border: 1px solid #d1d5db;
      border-radius: 0.5rem;
      font-size: 1rem;
      background: white;
      box-sizing: border-box;
      transition: border-color 0.2s, box-shadow 0.2s;
    }

    .form-input:focus {
      outline: none;
      border-color: #f59e0b;
      box-shadow: 0 0 0 3px rgba(245, 158, 11, 0.1);
    }

    .form-input:disabled {
      background: #f9fafb;
      color: #9ca3af;
      cursor: not-allowed;
    }

    .btn-primary {
      width: 100%;
      padding: 0.75rem;
      background: var(--colors-yellow40);
      color: var(--colors-yellow80);
      border: none;
      border-radius: 0.5rem;
      font-size: 1rem;
      font-weight: 500;
      cursor: pointer;
      transition: background-color 0.2s;
      margin-bottom: 0.5rem;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
    }

    .btn-primary:hover:not(:disabled) {
      background: #d97706;
    }
</style>
