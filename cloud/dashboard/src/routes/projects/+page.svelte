<script lang="ts">
    import { authState } from "$lib/auth.svelte";
    import { usersServiceClient } from "$lib/grpcClient";

    const user = authState().user;

    async function webAuthList() {
        console.log(authState());
        console.log(user);
        if (!user) return;

        const listCredentials = usersServiceClient
            .listWebauthnCredentials({
                id: user.id,
            });
        const response = await listCredentials.response;
        console.log("response", response);
    }
</script>

<svelte:head>
    <title>Projects | Scuffle</title>
</svelte:head>

Landing dashboard page. TODO: Mock up and add information about existing
projects and notable information

<button onclick={webAuthList}>
    click test
</button>
