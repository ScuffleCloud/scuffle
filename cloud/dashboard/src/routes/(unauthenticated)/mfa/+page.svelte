<script lang="ts">
    import TwoFactorPage from "$features/login-two-factor/two-factor-page.svelte";
    import LoginFooter from "$features/login/login-footer.svelte";
    import LoginHeader from "$features/login/login-header.svelte";
</script>

<div class="login-page-container">
    <LoginHeader />
    <TwoFactorPage />
    <LoginFooter />
</div>

<style>
    .login-page-container {
      min-height: 100vh;
      display: flex;
      align-items: center;
      justify-content: center;
      background-color: #f5f3f0;
      padding: 2rem;
      flex-direction: column;
    }
</style>
