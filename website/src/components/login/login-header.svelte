<script>
import ScuffleLogo from "$lib/images/scuffle-logo.svelte";
</script>

<div class="logo-container">
	<div class="logo-container-image">
		<ScuffleLogo />
	</div>
	scuffle
</div>

<style>
.logo-container {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--brown-800);
  font-size: 1.5rem;
  font-weight: 800;
  text-transform: uppercase;
  text-decoration: none;
  margin-bottom: 2rem;

  .logo-container-image {
    display: flex;
    align-items: center;
    justify-content: center;
    filter: drop-shadow(0px 2px 4px 0px rgb(0, 0, 0, 0.05));
  }
}
</style>
