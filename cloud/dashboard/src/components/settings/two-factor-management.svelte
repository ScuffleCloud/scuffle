<script lang="ts">
    import IconOverviewKey from "$lib/images/icon-overview-key.svelte";
    import IconShield from "$lib/images/icon-shield.svelte";

    type MfaMethod = {
        id: string;
        name: string;
        type: "TOTP" | "WEBAUTH";
        isPrimary?: boolean;
    };

    interface Props {
        enabled: boolean;
        methods: MfaMethod[];
        recoveryCodesGenerated: boolean;
        onToggle2FA: () => void;
        onAddMethod: () => void;
        onEditMethod: (id: string) => void;
        onDeleteMethod: (id: string) => void;
        onRegenerateCodes: () => void;
    }

    let {
        enabled,
        methods,
        recoveryCodesGenerated,
        onToggle2FA,
        onAddMethod,
        onEditMethod,
        onDeleteMethod,
        onRegenerateCodes,
    }: Props = $props();
</script>

<div class="tfa-management">
    <!-- Header -->
    <div class="section-header">
        <div class="header-icon">
            <IconShield />
        </div>
        <div class="header-content">
            <h2 class="section-title">2FA</h2>
            <span class="section-subtitle">(Two-factor authentication)</span>
        </div>
    </div>

    <!-- Main 2FA Card -->
    <div class="card">
        <div class="card-header">
            <div class="card-title-row">
                <h3 class="card-title">Two-factor authentication</h3>
                <span class="status-badge" class:enabled>
                    {enabled ? "Enabled" : "Disabled"}
                </span>
            </div>
        </div>

        <p class="card-description">
            Enables a second layer of security, by requiring at least two
            methods of authentication for signing in.
        </p>

        {#if enabled && methods.length > 0}
            <!-- Active Methods Section -->
            <div class="methods-section">
                <div class="methods-header">
                    <h4 class="methods-title">Active authentication methods</h4>
                    <button class="icon-button" onclick={onAddMethod}>
                        button
                    </button>
                </div>

                <div class="methods-list">
                    {#each methods as method (method.id)}
                        <div class="method-item">
                            <div class="method-icon">
                                <IconOverviewKey />
                            </div>
                            <div class="method-info">
                                <div class="method-name-row">
                                    <span class="method-name">{
                                        method.name
                                    }</span>
                                    {#if method.isPrimary}
                                        <span class="primary-badge"
                                        >Primary</span>
                                    {/if}
                                    <span class="method-type">{
                                        method.type
                                    }</span>
                                </div>
                            </div>
                            <button
                                class="icon-button"
                                onclick={() => onEditMethod(method.id)}
                            >
                                button
                            </button>
                        </div>
                    {/each}
                </div>

                <button class="add-method-button" onclick={onAddMethod}>
                    Add more methods
                </button>
            </div>
        {:else if enabled}
            <button class="action-button primary" onclick={onAddMethod}>
                Add Authentication Method
            </button>
        {:else}
            <button class="action-button primary" onclick={onToggle2FA}>
                Enable 2FA
            </button>
        {/if}
    </div>

    <!-- Recovery Codes Card -->
    {#if enabled}
        <div class="card">
            <div class="card-header">
                <div class="card-title-row">
                    <h3 class="card-title">2FA Recovery Codes</h3>
                    <span class="status-badge enabled">Enabled</span>
                </div>
            </div>

            <p class="card-description">
                Your recovery codes are private, store them securely.
                <br />
                If you generate new ones, the old recovery codes will no longer
                be valid.
            </p>

            <button class="action-button secondary" onclick={onRegenerateCodes}>
                {
                    recoveryCodesGenerated
                    ? "Regenerate codes"
                    : "Generate codes"
                }
            </button>
        </div>
    {/if}
</div>

<style>
    .tfa-management {
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
    }

    .section-header {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      padding: 0.5rem;
      background: var(--colors-gray50);
      border-radius: 8px;

      .header-icon {
        display: flex;
        align-items: center;
        justify-content: center;
      }

      .header-content {
        display: flex;
        align-items: center;
        gap: 0.5rem;

        .section-title {
          font-size: 1rem;
          font-weight: 700;
          color: var(--colors-brown90);
          margin: 0;
        }

        .section-subtitle {
          font-size: 1rem;
          font-weight: 500;
          color: var(--colors-brown50);
        }
      }
    }

    .card {
      background: var(--colors-gray20);
      border: 1px solid var(--colors-teal30);
      border-radius: 0.5rem;
      padding: 1rem;

      .card-header {
        margin-bottom: 0.75rem;

        .card-title-row {
          display: flex;
          align-items: center;
          gap: 0.75rem;

          .card-title {
            font-size: 1.125rem;
            font-weight: 700;
            color: var(--colors-brown90);
            margin: 0;
          }

          .status-badge {
            font-size: 0.875rem;
            font-weight: 700;
            padding: 0.25rem 0.5625rem;
            border-radius: 5.25rem;
            background: var(--colors-gray50);
            color: var(--colors-gray90);

            &.enabled {
              background: #dcfce7;
              color: #16a34a;
            }
          }
        }
      }

      .card-description {
        font-size: 1rem;
        font-weight: 500;
        line-height: 1.5rem;
        color: var(--colors-brown90);
        margin-bottom: 1rem;
      }
    }

    .methods-section {
      margin-top: 1rem;

      .methods-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.75rem;

        .methods-title {
          font-size: 1rem;
          font-weight: 600;
          color: var(--colors-brown70);
          margin: 0;
        }
      }

      .methods-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        margin-bottom: 1rem;

        .method-item {
          display: flex;
          align-items: center;
          gap: 0.75rem;
          padding: 0.75rem;
          background: var(--colors-gray30);
          border-radius: 0.5rem;

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

            .method-name-row {
              display: flex;
              align-items: center;
              gap: 0.5rem;
              flex-wrap: wrap;

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
            }
          }
        }
      }

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

        &:hover {
          background: var(--colors-gray50);
        }
      }
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

      &:hover {
        color: var(--colors-brown90);
      }
    }

    .action-button {
      padding: 0.5rem 0.75rem;
      border: none;
      border-radius: 0.5rem;
      font-size: 1rem;
      font-weight: 700;
      cursor: pointer;
      transition: all 0.2s;

      &.primary {
        background: var(--colors-yellow30);
        color: var(--colors-yellow90);

        &:hover {
          background: var(--colors-yellow40);
        }
      }

      &.secondary {
        background: #e5e7eb;
        color: #6b7280;

        &:hover {
          background: #d1d5db;
        }
      }
    }
</style>
