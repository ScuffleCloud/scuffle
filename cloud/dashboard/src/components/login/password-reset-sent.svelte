<script lang="ts">
    interface Props {
        email?: string;
        onBack?: () => void;
    }

    let { email, onBack }: Props = $props();
</script>

<h1 class="title">Check your email</h1>
<p class="subtitle">
    We've sent a password reset link to{#if email}
        <strong>{email}</strong>{:else}
        your email{/if}. Click the link in the email to reset your password.
</p>

<div class="info-box">
    <p class="info-text">
        <strong>Didn't receive the email?</strong><br />
        Check your spam folder or try again with a different email address.
    </p>
</div>

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

    .info-box {
      background-color: #f0f9ff;
      border: 1px solid #bae6fd;
      border-radius: 0.5rem;
      padding: 1rem;
      margin-bottom: 2rem;
    }

    .info-text {
      font-size: 0.875rem;
      color: #0369a1;
      margin: 0;
      line-height: 1.4;
    }
</style>
