<script lang="ts">
    interface Props {
        email: string;
    }

    let { email }: Props = $props();
</script>

<h1 class="title">Check your email for a magic link to continue!</h1>
<p class="subtitle">
    We've sent you an email to <strong>{email}</strong> with a magic link to
    verify your account.
</p>

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
</style>
