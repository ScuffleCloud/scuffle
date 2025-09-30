<script lang="ts">
    import SettingsCard from "$components/settings-card.svelte";
    import IconDots from "$lib/images/icon-dots.svelte";
    import IconOverviewKey from "$lib/images/icon-overview-key.svelte";

    export interface MfaMethod {
        id: string;
        name: string;
        type: "TOTP" | "WEBAUTH";
        isPrimary?: boolean;
    }

    interface Props {
        methods: MfaMethod[];
        onAddMethod: () => void;
        onEditMethod: (id: string) => void;
    }

    let { methods, onAddMethod, onEditMethod }: Props = $props();

    const enabled = $derived(methods.length > 0);
</script>

<SettingsCard
    title="Two-factor authentication"
    status={{
        label: enabled ? "Enabled" : "Disabled",
        variant: enabled ? "enabled" : "disabled",
    }}
    description="Enables a second layer of security, by requiring at least two methods of authentication for signing in."
>
    <div class="divider">
        Active authentication methods
        <div class="divider-line"></div>
    </div>
    {#if enabled && methods.length > 0}
        <div class="methods-list">
            {#each methods as method (method.id)}
                <div class="method-item">
                    <div class="method-icon">
                        <IconOverviewKey />
                    </div>
                    <div class="method-info">
                        <div class="method-name-row">
                            <span class="method-name">{method.name}</span>
                            {#if method.isPrimary}
                                <span class="primary-badge">Primary</span>
                            {/if}
                            <span class="method-type">{method.type}</span>
                        </div>
                    </div>
                    <button
                        class="icon-button"
                        onclick={() => onEditMethod(method.id)}
                        aria-label="Edit method"
                    >
                        <IconDots />
                    </button>
                </div>
            {/each}
        </div>
    {:else}
        <div class="no-active-methods">
            No active 2FA methods.
        </div>
    {/if}

    <div class="add-method-button-container">
        <button class="add-method-button" onclick={onAddMethod}>
            Add a method
        </button>
    </div>
</SettingsCard>

<style>
    .divider {
      display: flex;
      align-items: center;
      gap: 0.375rem;
      color: var(--text-3, #645c59);
      font-size: 0.875rem;
      font-style: normal;
      font-weight: 600;
      line-height: 1rem;
      padding: 0.75rem 0;

      .divider-line {
        flex: 1;
        height: 1px;
        background: var(--alpha-dark-10, rgba(24, 23, 22, 0.05));
      }
    }

    .methods-list {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      margin-bottom: 1rem;
    }

    .method-item {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      border-radius: 0.5rem;
    }

    .method-icon {
      width: 48px;
      height: 48px;
      display: flex;
      align-items: center;
      justify-content: center;
      background: white;
      border-radius: 0.5rem;
    }

    .method-info {
      flex: 1;
    }

    .method-name-row {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      flex-wrap: wrap;
    }

    .method-name {
      font-size: 1rem;
      font-weight: 600;
      color: var(--colors-brown90);
    }

    .primary-badge {
      font-size: 0.75rem;
      font-weight: 700;
      padding: 0.125rem 0.5rem;
      border-radius: 5.25rem;
      background: #fef3c7;
      color: #d97706;
    }

    .method-type {
      font-size: 0.875rem;
      font-weight: 600;
      padding: 0.125rem 0.5rem;
      border-radius: 0.25rem;
      background: var(--colors-gray50);
      color: var(--colors-brown70);
    }

    .icon-button {
      background: none;
      border: none;
      padding: 0.25rem;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--colors-brown70);
      transition: color 0.2s;
    }

    .icon-button:hover {
      color: var(--colors-brown90);
    }

    .add-method-button-container {
      display: flex;
      justify-content: center;
      align-items: center;
      margin-top: 1rem;

      .add-method-button {
        width: 100%;
        padding: 0.75rem;
        background: var(--colors-gray40);
        border: none;
        border-radius: 0.5rem;
        font-size: 1rem;
        font-weight: 600;
        color: var(--colors-brown90);
        cursor: pointer;
        transition: background 0.2s;
      }

      .add-method-button:hover {
        background: var(--colors-gray50);
      }
    }

    .no-active-methods {
      display: flex;
      padding: 0.75rem 1.5rem;
      justify-content: center;
      align-items: center;
      gap: 0.625rem;

      color: var(--text-3, #645c59);
      text-align: center;
      font-size: 1rem;
      font-style: normal;
      font-weight: 500;
      line-height: 1.5rem;
    }
</style>
