<script lang="ts">
    import LoginFormTitle from "./login-form-title.svelte";

    interface Props {
        email: string;
    }

    let { email }: Props = $props();
</script>

<LoginFormTitle title="Check your email for a magic link to continue!" />
<p class="subtitle">
    We've sent you an email to <strong>{email}</strong> with a magic link to
    verify your account.
</p>

<style>
    .subtitle {
      font-size: 0.875rem;
      color: #6b7280;
      line-height: 1.5;
    }
</style>
