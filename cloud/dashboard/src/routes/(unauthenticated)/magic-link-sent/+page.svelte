<script lang="ts">
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";

    // Only accessible through login flow
    onMount(() => {
        goto("/login");
    });
</script>
