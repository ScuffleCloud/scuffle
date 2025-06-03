<script lang="ts">
    import type { Component } from 'svelte';
    import type { Snippet } from 'svelte';
    import Switch from '$components/switch.svelte';

    export type BlockAction = {
        label?: string;
        variant?: 'primary' | 'secondary' | 'danger' | 'toggle';
        disabled?: boolean;
        onClick: () => void;
        isToggled?: boolean;
        enabledText?: string;
        disabledText?: string;
    };

    export type Card = {
        id: string;
        title: string;
        subtitle?: string;
        description?: string;
        status?: {
            label: string;
            variant: 'enabled' | 'disabled' | 'warning';
        };
        actions?: BlockAction[];
        customContent?: Snippet;
    };

    type Props = {
        title: string;
        subtitle?: string;
        icon: Component;
        cards: Card[];
    };

    const { title, subtitle, cards, icon }: Props = $props();

    function getStatusClass(variant: 'enabled' | 'disabled' | 'warning') {
        switch (variant) {
            case 'enabled':
                return 'status-enabled';
            case 'disabled':
                return 'status-disabled';
            case 'warning':
                return 'status-warning';
            default:
                return 'status-disabled';
        }
    }

    function getActionClass(variant: 'primary' | 'secondary' | 'danger' | 'toggle' = 'secondary') {
        switch (variant) {
            case 'primary':
                return 'action-primary';
            case 'secondary':
                return 'action-secondary';
            case 'danger':
                return 'action-danger';
            case 'toggle':
                return 'action-toggle';
            default:
                return 'action-secondary';
        }
    }

    function handleToggleChange(action: BlockAction, checked: boolean) {
        // Update the toggle state
        action.isToggled = checked;
        // Call the original onClick handler
        action.onClick();
    }
</script>

<div class="settings-block">
    <div class="block-header">
        <div class="block-icon">
            {#each [icon] as IconComponent}
                <IconComponent />
            {/each}
        </div>
        <div class="block-title-content">
            <h3 class="block-title">{title}</h3>
            {#if subtitle}
                <span class="block-subtitle">{subtitle}</span>
            {/if}
        </div>
    </div>
    <div class="cards-container">
        {#each cards as card}
            <div class="card">
                <div class="card-header">
                    <div class="card-title-section">
                        <h4 class="card-title">{card.title}</h4>
                        {#if card.status}
                            <span class="status-badge {getStatusClass(card.status.variant)}">
                                {card.status.label}
                            </span>
                        {/if}
                    </div>
                    {#if card.subtitle}
                        <p class="card-subtitle">{card.subtitle}</p>
                    {/if}
                </div>

                {#if card.description}
                    <p class="card-description">{card.description}</p>
                {/if}

                {#if card.customContent}
                    <div class="card-content">
                        {@render card.customContent()}
                    </div>
                {/if}

                {#if card.actions && card.actions.length > 0}
                    <div class="card-actions">
                        {#each card.actions as action}
                            {#if action.variant === 'toggle'}
                                <div class="toggle-action">
                                    <Switch
                                        checked={action.isToggled || false}
                                        disabled={action.disabled || false}
                                        showStateText={true}
                                        enabledText={action.enabledText || 'Enabled'}
                                        disabledText={action.disabledText || 'Disabled'}
                                        onchange={(checked) => handleToggleChange(action, checked)}
                                        size="medium"
                                    />
                                </div>
                            {:else}
                                <button
                                    class="action-button {getActionClass(action.variant)}"
                                    disabled={action.disabled}
                                    onclick={action.onClick}
                                >
                                    {action.label}
                                </button>
                            {/if}
                        {/each}
                    </div>
                {/if}
            </div>
        {/each}
    </div>
</div>

<style>
    .settings-block {
        background: var(--color-teal70);
        border-radius: 8px;
        padding: 0.25rem;
        max-width: 100%;

        .block-header {
            display: flex;
            align-items: center;

            .block-icon {
                display: flex;
                padding: 0.5rem;
            }

            .block-title-content {
                display: flex;
                align-items: center;
                gap: 0.5rem;

                .block-title {
                    font-size: 1rem;
                    font-weight: 700;
                    color: var(--color-brown90);
                    line-height: 1.5rem;
                }

                .block-subtitle {
                    font-size: 1rem;
                    font-weight: 500;
                    color: var(--color-brown50);
                    line-height: 1.5rem;
                }
            }
        }

        .cards-container {
            display: flex;
            flex-direction: column;
            gap: 0.25rem;

            .card {
                background: var(--color-teal30);
                border-radius: 0.5rem;
                padding: 1rem;
                border: 1px solid #e5e7eb;

                .card-header {
                    display: flex;
                    flex-direction: column;
                    align-items: flex-start;
                    gap: 0.25rem;
                    flex: 1 0 0;

                    .card-title-section {
                        display: flex;
                        align-items: center;
                        justify-content: space-between;
                        flex-wrap: wrap;
                        gap: 0.5rem;

                        .card-title {
                            color: var(--color-brown90);
                            font-size: 1.125rem;
                            font-weight: 700;
                            line-height: 1.5rem;
                        }
                    }

                    .card-subtitle {
                        color: var(--color-brown90);
                        font-size: 1rem;
                        font-weight: 500;
                        line-height: 1.5rem;
                    }
                }

                .status-badge {
                    padding: 0.25rem 0.75rem;
                    border-radius: 4px;
                    font-size: 0.875rem;
                    font-weight: 500;
                    white-space: nowrap;

                    &.status-enabled {
                        background: #dcfce7;
                        color: #16a34a;
                    }

                    &.status-disabled {
                        background: #fee2e2;
                        color: #dc2626;
                    }

                    &.status-warning {
                        background: #fef3c7;
                        color: #d97706;
                    }
                }

                .card-description {
                    color: #6b7280;
                    font-size: 0.875rem;
                    line-height: 1.5;
                    margin: 0 0 1rem 0;
                }

                .card-content {
                    margin-bottom: 1rem;
                }

                .card-actions {
                    display: flex;
                    gap: 0.75rem;
                    flex-wrap: wrap;
                    align-items: center;

                    .action-button {
                        border: none;
                        padding: 0.5rem 0.75rem;
                        border-radius: 0.5rem;
                        font-weight: 500;
                        cursor: pointer;
                        transition: all 0.2s;
                        min-width: fit-content;
                        color: var(--color-yellow90);
                        font-size: 1rem;
                        font-style: normal;
                        font-weight: 700;
                        line-height: 1.5rem;

                        &:disabled {
                            opacity: 0.5;
                            cursor: not-allowed;
                        }

                        &.action-primary {
                            background: var(--color-yellow30);

                            &:hover:not(:disabled) {
                                background: var(--color-yellow40);
                            }
                        }

                        &.action-secondary {
                            background: #e5e7eb;
                            color: #6b7280;

                            &:hover:not(:disabled) {
                                background: #d1d5db;
                            }
                        }

                        &.action-danger {
                            background: #ef4444;
                            color: white;

                            &:hover:not(:disabled) {
                                background: #dc2626;
                            }
                        }
                    }
                }
            }
        }
    }
</style>
