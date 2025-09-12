<script lang="ts">
    interface Props {
        onSubmit: (code: string) => Promise<void>;
        onBack?: () => void;
        isLoading?: boolean;
    }

    let { onSubmit, onBack, isLoading = false }: Props = $props();

    let inputs: HTMLInputElement[] = [];
    let code = $state(["", "", "", "", "", ""]);

    // Fix this because it's bad
    function handleInput(event: Event, index: number) {
        const target = event.target as HTMLInputElement;
        const value = target.value.replace(/[^0-9]/g, ""); // Only allow digits

        if (value.length > 1) {
            // Handle paste - distribute across inputs
            const digits = value.slice(0, 6).split("");
            code = [...digits, ...Array(6 - digits.length).fill("")];

            // Focus the next empty input or the last one
            const nextEmptyIndex = Math.min(digits.length, 5);
            inputs[nextEmptyIndex]?.focus();
        } else {
            // Single digit input
            code[index] = value;

            // Auto-advance to next input
            if (value && index < 5) {
                inputs[index + 1]?.focus();
            }
        }

        // Update the actual input values
        inputs.forEach((input, i) => {
            if (input) input.value = code[i];
        });
    }

    function handleKeydown(event: KeyboardEvent, index: number) {
        if (event.key === "Backspace" && !code[index] && index > 0) {
            // Move to previous input on backspace if current is empty
            inputs[index - 1]?.focus();
        } else if (event.key === "ArrowLeft" && index > 0) {
            inputs[index - 1]?.focus();
        } else if (event.key === "ArrowRight" && index < 5) {
            inputs[index + 1]?.focus();
        }
    }

    async function handleSubmit() {
        const fullCode = code.join("");
        if (fullCode.length === 6) {
            await onSubmit(fullCode);
        }
    }

    $effect(() => {
        // Focus first input on mount
        if (inputs[0]) {
            inputs[0].focus();
        }
    });

    $effect(() => {
        // Auto-submit when all 6 digits are entered
        const fullCode = code.join("");
        if (fullCode.length === 6 && !isLoading) {
            handleSubmit();
        }
    });
</script>

<div class="mfa-container">
    <div class="header">
        <h1 class="title">Authentication</h1>
    </div>

    <p class="subtitle">
        Enter the 6-digit code from your 2FA authenticator app below.
    </p>

    <div class="code-inputs">
        {#each Array(6) as _, index}
            <input
                bind:this={inputs[index]}
                type="text"
                inputmode="numeric"
                maxlength="6"
                class="code-input"
                class:filled={code[index]}
                disabled={isLoading}
                oninput={(e) => handleInput(e, index)}
                onkeydown={(e) => handleKeydown(e, index)}
                autocomplete="one-time-code"
            />
        {/each}
    </div>

    <button
        type="button"
        onclick={handleSubmit}
        class="continue-btn"
        disabled={isLoading || code.join("").length !== 6}
    >
        {#if isLoading}
            <div class="spinner"></div>
            Verifying...
        {:else}
            Continue
        {/if}
    </button>
</div>

<style>
    .mfa-container {
      max-width: 400px;
      margin: 0 auto;
      padding: 2rem;
      background: #faf9f8;
      border-radius: 1rem;
      text-align: center;
    }

    .header {
      display: flex;
      align-items: center;
      position: relative;
      margin-bottom: 1.5rem;
    }

    .back-button {
      position: absolute;
      left: 0;
      background: none;
      border: none;
      color: #6b7280;
      cursor: pointer;
      padding: 0.5rem;
      border-radius: 0.5rem;
      display: flex;
      align-items: center;
      justify-content: center;
    }

    .back-button:hover {
      background: #f3f4f6;
    }

    .title {
      flex: 1;
      font-size: 1.5rem;
      font-weight: 600;
      color: #1f2937;
      margin: 0;
    }

    .subtitle {
      color: #6b7280;
      font-size: 0.95rem;
      line-height: 1.5;
      margin: 0 0 2rem 0;
    }

    .code-inputs {
      display: flex;
      gap: 0.5rem;
      justify-content: center;
      margin-bottom: 2rem;
    }

    .code-input {
      width: 3rem;
      height: 3rem;
      border: 2px solid #e5e7eb;
      border-radius: 0.5rem;
      text-align: center;
      font-size: 1.25rem;
      font-weight: 600;
      color: #1f2937;
      background: white;
      transition: all 0.2s;
      outline: none;
    }

    .code-input:focus {
      border-color: #f59e0b;
      box-shadow: 0 0 0 3px rgba(245, 158, 11, 0.1);
    }

    .code-input.filled {
      border-color: #10b981;
      background: #f0fdf4;
    }

    .code-input:disabled {
      background: #f9fafb;
      color: #9ca3af;
      cursor: not-allowed;
    }

    .continue-btn {
      width: 100%;
      padding: 0.875rem;
      background: #f59e0b;
      color: white;
      border: none;
      border-radius: 0.5rem;
      font-size: 1rem;
      font-weight: 600;
      cursor: pointer;
      transition: background-color 0.2s;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
    }

    .continue-btn:hover:not(:disabled) {
      background: #d97706;
    }

    .continue-btn:disabled {
      background: #d1d5db;
      cursor: not-allowed;
    }

    .spinner {
      width: 16px;
      height: 16px;
      border: 2px solid rgba(255, 255, 255, 0.3);
      border-top: 2px solid white;
      border-radius: 50%;
      animation: spin 1s linear infinite;
    }

    @keyframes spin {
      0% {
        transform: rotate(0deg);
      }
      100% {
        transform: rotate(360deg);
      }
    }

    @media (max-width: 480px) {
      .mfa-container {
        padding: 1.5rem;
        margin: 1rem;
      }

      .code-input {
        width: 2.5rem;
        height: 2.5rem;
        font-size: 1.1rem;
      }

      .code-inputs {
        gap: 0.375rem;
      }
    }
</style>
