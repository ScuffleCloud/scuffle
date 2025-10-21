<script lang="ts">
    const { children, class: className = "", ...props } = $props();
</script>

<div class="card {className}" {...props}>
    {@render children()}
</div>

<style>
    .card {
      border-radius: 1.25rem;
      padding: 2.75rem;
      width: 400px;
      box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05);
      border: 1px solid var(--colors-gray50);
      background-color: var(--colors-gray20);
      text-align: center;
      margin-bottom: 2rem;
    }
</style>
