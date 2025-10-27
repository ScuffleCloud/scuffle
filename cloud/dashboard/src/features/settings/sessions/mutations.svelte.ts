import { queryKeys } from "$lib/consts";
import { sessionsServiceClient } from "$lib/grpcClient";
import { withRpcErrorHandling } from "$lib/utils";
import type { UserSession } from "@scufflecloud/proto/scufflecloud/core/v1/sessions.js";
import { createMutation, type QueryClient, useQueryClient } from "@tanstack/svelte-query";

type RemoveSessionType = {
    id: string;
    deviceFingerprint: Uint8Array;
};

export function useRemoveSession(userId: string | undefined) {
    const queryClient: QueryClient = useQueryClient();

    return createMutation<void, Error, RemoveSessionType>(() => ({
        mutationFn: async ({ id, deviceFingerprint }: RemoveSessionType) =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");

                const call = sessionsServiceClient.invalidateUserSession({ userId: id, deviceFingerprint });

                const status = await call.status;
                if (status.code !== "OK") {
                    throw new Error(status.detail || "Failed to remove session");
                }
            }),
        onSuccess: (_data, { id }) => {
            // queryClient.setQueryData<UserSession[]>(
            //     [queryKeys.sessions(userId!)],
            //     (old: UserSession[] | undefined) => old?.filter(session => session.deviceFingerprint !== deviceFingerprint.toString()),
            // );
        },
    }));
}
