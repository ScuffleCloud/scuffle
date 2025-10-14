<script lang="ts">
    import Navbar from "$lib/components/left-nav/navbar.svelte";
    import RightNav from "$lib/components/right-nav/right-nav.svelte";
    import TopNav from "$lib/components/top-nav/top-nav.svelte";

    const { children } = $props();
</script>

<!-- Authenticated -->
<div class="app">
    <Navbar />
    <main>
        <TopNav />
        <div class="content">
            <div class="main-content">
                {@render children()}
            </div>
            <RightNav />
        </div>
    </main>
</div>

<style>
    .app {
      position: relative;
      display: flex;
      min-height: 100vh;
      background-color: var(--colors-light100);
    }

    main {
      position: relative;
      flex: 1;
      display: flex;
      flex-direction: column;
      box-sizing: border-box;
      width: 100%;
      /* So content resizes properly */
      min-width: 0;

      .content {
        display: flex;
        flex-direction: row;
        padding: 2rem;

        .main-content {
          flex: 1;
          width: 100%;
          /* So content resizes properly */
          min-width: 0;
        }
      }
    }
</style>
