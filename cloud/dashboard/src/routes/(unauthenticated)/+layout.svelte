<script lang="ts">
    const { children } = $props();
</script>

<!-- Unauthenticated -->
<div class="login-page-container">
    {@render children()}
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
