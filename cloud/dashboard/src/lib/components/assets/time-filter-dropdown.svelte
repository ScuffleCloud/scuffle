<script lang="ts">
    import IconDate from "$lib/images/icon-date.svelte";
    import { createDropdownMenu, melt } from "@melt-ui/svelte";

    interface TimeFilter {
        id: string;
        name: string;
        icon?: string;
    }

    interface Props {
        selectedFilter?: string;
        onFilterChange: (filterId: string) => void;
    }

    const { selectedFilter = "latest", onFilterChange }: Props =
        $props();

    const timeFilters: TimeFilter[] = [
        { id: "latest", name: "Latest" },
        { id: "last30days", name: "Last 30 days" },
        { id: "last3months", name: "Last 3 months" },
    ];

    const {
        elements: { trigger, menu, item },
        states: { open },
    } = createDropdownMenu({
        forceVisible: true,
    });

    const selectedFilterData = $derived(
        timeFilters.find(filter => filter.id === selectedFilter)
            || timeFilters[0],
    );

    function handleFilterSelect(filterId: string) {
        onFilterChange(filterId);
    }
</script>

<div class="time-filter-container">
    <button class="time-filter-trigger" use:melt={$trigger}>
        <div class="selected-filter">
            <span class="filter-icon"><IconDate /></span>
            <span class="filter-name">{selectedFilterData.name}</span>
        </div>
        <svg
            class="dropdown-arrow"
            class:rotated={$open}
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
        >
            <path
                d="M4 6L8 10L12 6"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
            />
        </svg>
    </button>

    {#if $open}
        <div class="dropdown-menu" use:melt={$menu}>
            {#each timeFilters as filter (filter.id)}
                <div
                    class="menu-item"
                    class:selected={filter.id === selectedFilter}
                    use:melt={$item}
                    onclick={() => handleFilterSelect(filter.id)}
                >
                    <div class="menu-item-content">
                        <span class="filter-icon">{filter.icon}</span>
                        <span class="filter-name">{filter.name}</span>
                        {#if filter.id === selectedFilter}
                            <svg
                                class="check-icon"
                                width="16"
                                height="16"
                                viewBox="0 0 16 16"
                                fill="none"
                            >
                                <path
                                    d="M13.5 4.5L6 12L2.5 8.5"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                />
                            </svg>
                        {/if}
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>

<style>
    .time-filter-container {
      position: relative;
      display: inline-block;
    }

    .time-filter-trigger {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 0.5rem;
      padding: 0.5rem 0.75rem;
      background: #ffffff;
      border: 1px solid #e5e5e5;
      border-radius: 0.5rem;
      font-size: 0.875rem;
      font-weight: 500;
      color: #374151;
      cursor: pointer;
      transition: all 0.2s ease;
      min-width: 140px;
      height: 100%;

      &:hover {
        background-color: #f9fafb;
        border-color: #d1d5db;
      }

      &:focus {
        outline: none;
        box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
      }

      .selected-filter {
        display: flex;
        align-items: center;
        gap: 0.5rem;

        .filter-icon {
          font-size: 0.875rem;
        }

        .filter-name {
          white-space: nowrap;
        }
      }

      .dropdown-arrow {
        color: #6b7280;
        transition: transform 0.2s ease;
        flex-shrink: 0;

        &.rotated {
          transform: rotate(180deg);
        }
      }
    }

    .dropdown-menu {
      z-index: 50;
      background: #ffffff;
      border: 1px solid #e5e5e5;
      border-radius: 0.5rem;
      padding: 0.5rem;
      animation: fadeIn 0.15s ease-out;
    }

    .menu-item {
      padding: 0.5rem 0.75rem;
      cursor: pointer;
      border-radius: 0.375rem;
      transition: background-color 0.15s ease;

      &:hover {
        background-color: #f3f4f6;
      }

      &.selected {
        background-color: #eff6ff;
      }

      .menu-item-content {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        justify-content: space-between;

        .filter-icon {
          font-size: 0.875rem;
        }

        .filter-name {
          font-size: 0.875rem;
          font-weight: 400;
          flex: 1;
          text-align: left;
        }

        .check-icon {
          color: #2563eb;
          flex-shrink: 0;
        }
      }
    }

    @keyframes fadeIn {
      from {
        opacity: 0;
        transform: translateY(-4px);
      }
      to {
        opacity: 1;
        transform: translateY(0);
      }
    }
</style>
