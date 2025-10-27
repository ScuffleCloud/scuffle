// Use in order to invalidate queries from the authstate, since it is initialized before the query client is set
import type { QueryClient } from "@tanstack/svelte-query";

let queryClient: QueryClient | null = null;

export function getAuthQueryClient() {
    return queryClient;
}

export function setAuthQueryClient(client: QueryClient) {
    queryClient = client;
}
