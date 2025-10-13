<script lang="ts">
    import TwoFactorPage from "$components/login-two-factor/two-factor-page.svelte";
    import LoginFooter from "$components/login/login-footer.svelte";
    import LoginHeader from "$components/login/login-header.svelte";
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
