<script lang="ts">
    import InlineNotification from "$lib/components/inline-notification.svelte";
    import IconCopy from "$lib/images/icon-copy.svelte";
    import { TotpCredential } from "@scufflecloud/proto/scufflecloud/core/v1/users_service.js";
    import type { CreateMutationResult } from "@tanstack/svelte-query";
    import { onMount } from "svelte";
    import type { CreateTotpCredentialMutationResponse } from "./credentialMutations.svelte";

    interface Props {
        createMutation: CreateMutationResult<
            CreateTotpCredentialMutationResponse,
            Error,
            void,
            unknown
        >;
        completeMutation: CreateMutationResult<
            TotpCredential,
            Error,
            { code: string },
            unknown
        >;
        validateCode: (code: string) => void;
    }

    let { createMutation, completeMutation, validateCode }: Props =
        $props();

    // So loading state does not flicker when this component is mounted
    let loading = $state(false);

    onMount(() => {
        loading = true;
    });

    $effect(() => {
        if (createMutation.data) {
            loading = false;
        }
    });

    let verificationCode = $state("");
</script>

{#if createMutation.isError}
    <InlineNotification
        type="error"
        message={createMutation.error?.message || "Failed to generate QR code"}
    />
{:else if loading}
    <div class="skeleton-container">
        <div class="qr-skeleton skeleton"></div>
        <div class="text-skeleton skeleton"></div>
    </div>
{:else if createMutation.data}
    {@const { qrCodeUrl, secretKey } = createMutation.data}
    <div class="totp-scan">
        <p class="description">
            <strong>Step 1. </strong> Scan or paste the QR code below to your
            preferred authenticator app.
        </p>

        <div class="secret-key-container">
            <code>{secretKey}</code>
            <button
                class="copy-button"
                onclick={() => navigator.clipboard.writeText(secretKey)}
                aria-label="Copy secret key"
            >
                <IconCopy />
            </button>
        </div>
        <div class="qr-container">
            <img
                src={qrCodeUrl}
                alt="QR Code for 2FA setup"
                class="qr-code"
            />
        </div>

        <div class="verification-section">
            <p class="description">
                <strong>Step 2. </strong> Enter the 6-digit code from your
                authenticator app to verify
            </p>

            <input
                type="text"
                id="verification-code"
                class="verification-input"
                bind:value={verificationCode}
                disabled={completeMutation.isPending}
            />
            {#if completeMutation.isError}
                <InlineNotification
                    type="error"
                    message={completeMutation.error?.message
                    || "Failed to verify code"}
                />
            {/if}
            <button
                class="next-button"
                onclick={() => validateCode(verificationCode)}
                disabled={completeMutation.isPending
                || verificationCode.length < 6}
            >
                Continue
            </button>
        </div>
    </div>
{/if}

<style>
    .totp-scan {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 1.5rem;
      padding: 0.5rem;
    }

    .description {
      font-size: 0.9375rem;
      color: rgb(82, 82, 82);
      margin: 0;
      line-height: 1.5;
    }

    .qr-container {
      background: white;
      padding: 0.25rem;
      border-radius: 0.5rem;
    }

    .qr-code {
      display: block;
      width: 200px;
      height: 200px;
    }

    .secret-key-container {
      display: flex;
      align-items: center;
      justify-content: space-between;
      background: white;
      padding: 1rem 1.5rem;
      border-radius: 0.75rem;
      border: 1px solid rgb(228, 228, 231);
      width: 100%;
      gap: 1rem;
      box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);

      code {
        font-size: 0.75rem;
        color: #18181b;
        flex: 1;
        background: transparent;
        border: none;
        padding: 0;
        word-break: break-all;
      }

      .copy-button {
        display: flex;
        align-items: center;
        justify-content: center;
        background: transparent;
        border: none;
        cursor: pointer;
        padding: 0.5rem;
        color: #71717a;
        border-radius: 0.375rem;
        transition: all 0.2s;
        flex-shrink: 0;
      }

      .copy-button:hover {
        background: rgb(244, 244, 245);
        color: #18181b;
      }

      .copy-button:active {
        transform: scale(0.95);
      }
    }

    .skeleton-container {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 1rem;
      padding: 1.5rem;

      .qr-skeleton {
        width: 200px;
        height: 200px;
        border-radius: 0.5rem;
      }

      .text-skeleton {
        width: 250px;
        height: 2.5rem;
        border-radius: 0.25rem;
      }
    }

    .verification-section {
      display: flex;
      flex-direction: column;
      align-items: stretch;
      gap: 1rem;
      width: 100%;
      max-width: 400px;
    }

    .verification-input {
      width: 100%;
      padding: 0.75rem 1rem;
      border: 1px solid #d1d5db;
      border-radius: 0.5rem;
      font-size: 1rem;
      background: white;
      box-sizing: border-box;
      transition: border-color 0.2s, box-shadow 0.2s;
      text-align: center;
    }

    .verification-input:focus {
      outline: none;
      border-color: #f59e0b;
      box-shadow: 0 0 0 3px rgba(245, 158, 11, 0.1);
    }

    .verification-input:disabled {
      background: #f9fafb;
      color: #9ca3af;
      cursor: not-allowed;
    }

    .verification-input::placeholder {
      letter-spacing: 0.5rem;
      color: #d1d5db;
    }

    .next-button {
      width: 100%;
      padding: 0.75rem;
      background: var(--colors-yellow40);
      color: var(--colors-yellow80);
      border: none;
      border-radius: 0.5rem;
      font-size: 1rem;
      font-weight: 500;
      cursor: pointer;
      transition: background-color 0.2s;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
    }

    .next-button:hover:not(:disabled) {
      background: #d97706;
    }

    .next-button:disabled {
      background: #f9fafb;
      color: #9ca3af;
      cursor: not-allowed;
    }
</style>
