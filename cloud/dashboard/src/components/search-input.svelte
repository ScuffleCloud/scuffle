<script lang="ts">
    import IconSearch from "$lib/images/icon-search.svelte";

    type Props = {
        placeholder?: string;
        value?: string;
        onInput?: (value: string) => void;
    };

    const {
        placeholder = "Search...",
        value = $bindable(""),
        onInput,
    }: Props = $props();

    function handleInput(event: Event) {
        const target = event.target as HTMLInputElement;
        onInput?.(target.value);
    }
</script>

<div class="search-container">
    <div class="search-icon">
        <IconSearch />
    </div>
    <input
        type="text"
        {placeholder}
        {value}
        oninput={handleInput}
    />
</div>

<style>
    .search-container {
      display: flex;
      justify-content: space-between;
      margin-bottom: 1.5rem;
      position: relative;
      flex: 1;

      .search-icon {
        position: absolute;
        left: 0.75rem;
        top: 50%;
        transform: translateY(-50%);
        display: flex;
        align-items: center;
        z-index: 1;
      }

      input {
        width: 100%;
        padding: 0.75rem 1rem 0.75rem 2.5rem;
        background-color: inherit;
        border: 1px solid var(--alpha-dark-20, rgba(24, 23, 22, 0.12));
        border-radius: 0.5rem;
        font-size: 1rem;

        &:focus {
          outline: none;
          box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.1);
        }
      }
    }
</style>
