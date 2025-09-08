<!-- For testing https://developers.cloudflare.com/turnstile/troubleshooting/testing/ -->

<script lang="ts">
    import { PUBLIC_TURNSTILE_SITE_KEY } from "$env/static/public";
    import { Turnstile } from "svelte-turnstile";
    import type { TurnstileError } from "./types";

    let showTurnstileOverlay = $state(false);
    export { showTurnstileOverlay };
    let reset: () => void = $state(() => {});

    let token: string | undefined = undefined;

    let handleTurnstileCallback = (event: CustomEvent) => {
        token = event.detail.token;
    };

    export const getToken = async (): Promise<string> => {
        return new Promise((resolve, reject) => {
            if (token) {
                resolve(token);
                token = undefined;
                reset();
                return;
            } else {
                // Otherwise reset as must be blocked or interaction needed
                reset();
            }
            showTurnstileOverlay = true;

            // Otherwise wait for token to be set and resolve
            const originalCallback = handleTurnstileCallback;
            handleTurnstileCallback = (event: CustomEvent) => {
                // Reject if captcha failed
                if (!event.detail.token) {
                    reject(
                        {
                            message: "Captcha failed",
                            wasCaptcha: true,
                        } satisfies TurnstileError,
                    );
                } else {
                    resolve(event.detail.token);
                }
                token = undefined;

                // Reset callback
                handleTurnstileCallback = originalCallback;
                reset();
                showTurnstileOverlay = false;
            };
        });
    };
</script>

<div
    class={["overlay", { hidden: !showTurnstileOverlay }]}
    data-testid="turnstile-overlay"
>
    <div class="turnstile-box">
        <p>One more step before you proceed...</p>
        <div class="turnstile-container">
            <Turnstile
                siteKey={PUBLIC_TURNSTILE_SITE_KEY}
                on:callback={(event) => handleTurnstileCallback(event)}
                on:after-interactive={() => {
                    showTurnstileOverlay = false;
                }}
                on:error={(event) => {
                    handleTurnstileCallback(event);
                }}
                bind:reset
            />
        </div>
    </div>
</div>

<style>
    .overlay {
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background-color: rgba(0, 0, 0, 0.7);
      justify-content: center;
      align-items: center;
      z-index: 100;

      & p {
        color: white;
        text-align: center;
        margin-bottom: 1rem;
        margin-top: 40vh;
      }

      & .turnstile-container {
        display: flex;
        flex-wrap: nowrap;
        align-items: center;
        justify-content: center;
      }
    }

    .hidden {
      opacity: 0;
      visibility: hidden;
    }
</style>
