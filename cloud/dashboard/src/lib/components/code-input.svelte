<script lang="ts">
    import { PinInput } from "melt/builders";

    interface Props {
        value?: string;
        disabled?: boolean;
        placeholder?: string;
        maxLength?: number;
        type?: "numeric" | "alphanumeric" | "text";
    }

    let {
        value = $bindable(""),
        disabled = false,
        placeholder = "-",
        maxLength = 6,
        type = "numeric",
    }: Props = $props();

    const pinInput = new PinInput({
        maxLength,
        type,
        placeholder,
        disabled: () => disabled,
        value: () => value,
        onValueChange: (newValue) => {
            value = newValue;
        },
    });
</script>

<div {...pinInput.root} class="pin-input-root">
    {#each pinInput.inputs as input, index (`pin-input-${index}`)}
        <input {...input} class="pin-input" />
    {/each}
</div>

<style>
    .pin-input-root {
      display: flex;
      gap: 0.5rem;
      justify-content: center;
      margin-bottom: 1.5rem;
    }

    .pin-input {
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

    .pin-input:focus {
      border-color: #f59e0b;
      box-shadow: 0 0 0 3px rgba(245, 158, 11, 0.1);
    }

    .pin-input:disabled {
      background: #f9fafb;
      color: #9ca3af;
      cursor: not-allowed;
    }

    @media (max-width: 480px) {
      .pin-input {
        width: 2.5rem;
        height: 2.5rem;
        font-size: 1.1rem;
      }

      .pin-input-root {
        gap: 0.375rem;
      }
    }
</style>
