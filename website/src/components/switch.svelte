<script lang="ts">
    import { createSwitch, melt } from '@melt-ui/svelte';

    interface Props {
        checked?: boolean;
        disabled?: boolean;
        required?: boolean;
        name?: string;
        value?: string;
        label?: string;
        showStateText?: boolean;
        enabledText?: string;
        disabledText?: string;
        size?: 'small' | 'medium' | 'large';
        onchange?: (checked: boolean) => void;
    }

    let {
        checked = $bindable(false),
        disabled = false,
        required = false,
        name = '',
        value = 'on',
        label = '',
        showStateText = false,
        enabledText = 'Enabled',
        disabledText = 'Disabled',
        size = 'medium',
        onchange,
    }: Props = $props();

    const {
        elements: { root, input },
        states: { checked: checkedState },
        options,
    } = createSwitch({
        defaultChecked: checked,
        disabled,
        required,
        name,
        value,
        onCheckedChange: ({ next }) => {
            checked = next;
            onchange?.(next);
            return next;
        },
    });

    // Size configurations
    const sizeConfig = {
        small: {
            width: '2rem',
            height: '1rem',
            thumbSize: '0.75rem',
            padding: '0.125rem',
        },
        medium: {
            width: '2.75rem',
            height: '1.5rem',
            thumbSize: '1.25rem',
            padding: '0.125rem',
        },
        large: {
            width: '3.5rem',
            height: '2rem',
            thumbSize: '1.75rem',
            padding: '0.125rem',
        },
    };

    const currentSize = $derived(sizeConfig[size]);
</script>

<div class="switch-container">
    {#if label}
        <label class="switch-label" for={name || 'switch'}>
            {label}
        </label>
    {/if}

    <div class="switch-wrapper">
        <button
            use:melt={$root}
            class="switch-root"
            class:checked={$checkedState}
            class:disabled
            id={name || 'switch'}
            style="
          --switch-width: {currentSize.width};
          --switch-height: {currentSize.height};
          --thumb-size: {currentSize.thumbSize};
          --padding: {currentSize.padding};
          "
        >
            <span class="switch-thumb" class:checked={$checkedState}></span>
        </button>

        <input use:melt={$input} />

        {#if showStateText}
            <span class={`state-text ${$checkedState ? 'checked' : 'unchecked'}`}>
                {$checkedState ? enabledText : disabledText}
            </span>
        {/if}
    </div>
</div>

<style>
    .switch-container {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;

        .switch-wrapper {
            display: flex;
            align-items: center;
            gap: 0.75rem;
        }

        .switch-label {
            font-size: 0.875rem;
            font-weight: 500;
            color: #374151;
            cursor: pointer;
            user-select: none;
        }

        .switch-root {
            position: relative;
            width: var(--switch-width);
            height: var(--switch-height);
            background: var(--colors-gray40);
            box-shadow: 0px 0.25px 2px 0px var(--colors-gray20);
            border: none;
            border-radius: 100rem;
            cursor: pointer;
            transition: all 0.2s ease;
            outline: none;
            padding: 0.125rem;

            &.checked {
                box-shadow: 0px 0.25px 2px 0px var(--colors-yellow30);
                background-color: var(--colors-yellow40);

                .switch-thumb {
                    transform: translateX(
                        calc(var(--switch-width) - var(--thumb-size) - 2 * var(--padding))
                    );
                }
            }

            &.disabled {
                opacity: 0.5;
                cursor: not-allowed;
            }

            &:not(.disabled) {
                &:hover {
                    filter: brightness(0.95);
                }

                &:active {
                    transform: scale(0.98);
                }
            }

            .switch-thumb {
                position: absolute;
                top: var(--padding);
                left: var(--padding);
                width: var(--thumb-size);
                height: var(--thumb-size);
                background-color: white;
                border-radius: 100rem;
                transition: transform 0.2s ease;
                box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
                border: 1px solid #fff;

                &.checked {
                    box-shadow: 0px 0.25px 2px 0px #ffa002;
                }
            }
        }

        .state-text {
            font-size: 0.875rem;
            font-weight: 700;
            line-height: 1.5rem;
            user-select: none;
            display: flex;
            padding: 0.125rem 0.5rem;
            justify-content: center;
            align-items: center;
            border-radius: 0.5rem;

            &.checked {
                color: var(--colors-yellow80);
                border: 1px solid var(--colors-yellow30);
                background: var(--colors-yellow10);
            }

            &.unchecked {
                color: var(--colors-gray80);
                border: 1px solid var(--colors-gray30);
                background: var(--colors-gray10);
            }
        }
    }
</style>
