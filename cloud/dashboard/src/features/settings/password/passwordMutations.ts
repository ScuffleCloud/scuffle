import { usersServiceClient } from "$lib/grpcClient";
import { withRpcErrorHandling } from "$lib/utils";
import { createMutation } from "@tanstack/svelte-query";

type UpdatePasswordParams = {
    currentPassword: string;
    newPassword: string;
    confirmPassword: string;
};

type SetPasswordParams = {
    password: string;
    confirmPassword: string;
};

function validateBasicPassword(password: string, confirmPassword: string): void | Error {
    if (password.length < 8) {
        throw new Error("Password must be at least 8 characters");
    }
    if (password.length > 40) {
        throw new Error("Password must be less than 40 characters");
    }
    if (password.length < 15 && !/\d/.test(password)) {
        throw new Error("Password must contain at least one number");
    }
    if (password.length < 15 && !/[a-z]/.test(password)) {
        throw new Error("Password must contain at least one lowercase letter");
    }
    if (password !== confirmPassword) {
        throw new Error("New passwords do not match");
    }
}

export function useUpdatePassword(userId: string | undefined) {
    return createMutation(() => ({
        mutationFn: ({ currentPassword, newPassword, confirmPassword }: UpdatePasswordParams) =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");

                if (!currentPassword || !newPassword || !confirmPassword) {
                    throw new Error("All fields are required");
                }

                validateBasicPassword(newPassword, confirmPassword);

                return await usersServiceClient.updateUser({
                    id: userId,
                    password: {
                        currentPassword: currentPassword,
                        newPassword: newPassword,
                    },
                }).response;
            }),
    }));
}

export function useSetPassword(userId: string | undefined) {
    return createMutation(() => ({
        mutationFn: ({ password, confirmPassword }: SetPasswordParams) =>
            withRpcErrorHandling(async () => {
                if (!userId) throw new Error("User not authenticated");
                validateBasicPassword(password, confirmPassword);

                return await usersServiceClient.updateUser({
                    id: userId,
                    password: {
                        newPassword: password,
                    },
                }).response;
            }),
    }));
}
