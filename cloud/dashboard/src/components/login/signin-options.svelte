<script lang="ts">
    import type { LoginMode } from "$components/streams/types";
    import { useAuth } from "$lib/auth.svelte";
    import { sessionsServiceClient } from "$lib/grpcClient";
    import IconGoogle from "$lib/images/icon-google.svelte";
    import IconLoginKey from "$lib/images/icon-login-key.svelte";

    interface Props {
        onModeChange: (mode: LoginMode) => void;
        isLoading?: boolean;
    }

    $effect(() => {
        const urlParams = new URLSearchParams(window.location.search);
        const code = urlParams.get("code");
        const state = urlParams.get("state");

        if (code && state) {
            handleGoogleCallback(code, state);
        }
    });

    const auth = useAuth();

    let { onModeChange, isLoading = false }: Props = $props();

    // Probably move this to a generic function later
    async function handleGoogleLogin(): Promise<void> {
        try {
            const device = await auth.getDeviceOrInit();
            console.log(JSON.stringify(device));
            const call = sessionsServiceClient.loginWithGoogle({
                device,
            });
            const status = await call.status;

            if (status.code === "OK") {
                const response = await call.response;
                window.location.href = response.authorizationUrl;
            } else {
                console.error("Google login failed:", status.detail);
            }
        } catch (error) {
            console.error("Login with Google error:", error);
        }
    }

    async function handleGoogleCallback(
        code: string,
        state: string,
    ): Promise<void> {
        isLoading = true;

        try {
            // Get device public key again for the completion request
            const device = await auth.getDeviceOrInit();
            console.log(JSON.stringify(device));

            const call = sessionsServiceClient.completeLoginWithGoogle({
                code,
                state,
                device,
            });

            const status = await call.status;
            console.log(status);

            if (status.code === "OK") {
                const response = await call.response;

                console.log("response");
                console.log(response);

                // TODO: Revisit after "this call is not implemented yet error" resolved
                // This might be enough to handle the login though and will reroute from layout?
                if (response.newUserSessionToken) {
                    await auth.handleNewUserSessionToken(
                        response.newUserSessionToken,
                    );
                }
            } else {
                console.error(
                    "Google login completion failed:",
                    status.detail,
                );
            }
        } catch (error) {
            console.error("Google login completion error:", error);
        } finally {
            isLoading = false;
        }
    }

    function handlePasskeyLogin() {
        onModeChange("passkey");
    }
</script>

<div class="divider">OR</div>
<button
    type="button"
    onclick={async () => console.log(JSON.stringify(await auth.getDeviceOrInit()))}
    class="btn-social google"
>
    TEST auth
</button>
<button
    type="button"
    onclick={handleGoogleLogin}
    class="btn-social google"
    disabled={isLoading}
>
    <IconGoogle />
    Continue with Google
</button>

<button
    type="button"
    onclick={handlePasskeyLogin}
    class="btn-social passkey"
    disabled={isLoading}
>
    <IconLoginKey />
    Continue with Passkey
</button>

<style>
    .divider {
      display: flex;
      align-items: center;
      margin: 2rem 0;
      color: #9ca3af;
      font-size: 0.875rem;
      text-transform: uppercase;
    }

    .divider::before,
    .divider::after {
      content: "";
      flex: 1;
      height: 1px;
      background: #d1d5db;
    }

    .divider::before {
      margin-right: 0.325rem;
    }

    .divider::after {
      margin-left: 0.325rem;
    }

    .btn-social {
      width: 100%;
      padding: 0.75rem;
      background: white;
      color: #374151;
      border: 1px solid #d1d5db;
      cursor: pointer;
      margin-bottom: 0.5rem;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
      border-radius: 0.5rem;
    }

    .btn-social:hover:not(:disabled) {
      background: #f9fafb;
      border-color: #9ca3af;
    }

    .btn-social:disabled {
      background: white;
      color: #9ca3af;
      cursor: not-allowed;
    }

    .passkey-link {
      text-decoration: none;
    }
</style>
