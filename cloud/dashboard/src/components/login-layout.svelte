<script lang="ts">
    import LoginFooter from "./login/login-footer.svelte";
    import LoginHeader from "./login/login-header.svelte";
    import LoginPage from "./login/login-page.svelte";
</script>

<div class="login-page-container">
    <LoginHeader />
    <LoginPage />
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
