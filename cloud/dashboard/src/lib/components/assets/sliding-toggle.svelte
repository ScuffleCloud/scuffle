<script lang="ts">
    import type { Component } from "svelte";

    type Props = {
        leftLabel: Component;
        rightLabel: Component;
        value: "left" | "right";
        onToggle: () => void;
    };

    let {
        leftLabel,
        rightLabel,
        value = "left",
        onToggle = () => {},
    }: Props = $props();

    function handleToggle() {
        value = value === "left" ? "right" : "left";
        onToggle();
    }

    const LeftLabel = leftLabel;
    const RightLabel = rightLabel;
</script>

<div class="toggle-container">
    <div class="toggle-track">
        <div
            class="slider-bg"
            class:slide-right={value === "right"}
        >
        </div>
        <button
            class="toggle-button"
            class:active={value === "left"}
            onclick={handleToggle}
            type="button"
        >
            <LeftLabel />
        </button>
        <button
            class="toggle-button"
            class:active={value === "right"}
            onclick={handleToggle}
            type="button"
        >
            <RightLabel />
        </button>
    </div>
</div>
<style>
    .toggle-container {
      display: inline-block;
      height: 100%;
    }

    .toggle-track {
      position: relative;
      display: flex;
      background: #f1f5f9;
      border-radius: 0.5rem;
      padding: 0.25rem;
      border: 1px solid #e2e8f0;
      flex: 1;
      height: 100%;
      align-items: center;
    }

    .slider-bg {
      position: absolute;
      top: 0.25rem;
      left: 0.25rem;
      width: calc(50% - 0.25rem);
      height: calc(100% - 0.5rem);
      background: white;
      border-radius: 0.5rem;
      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
      transition: transform 0.2s ease-in-out;
      z-index: 1;
    }

    .slider-bg.slide-right {
      transform: translateX(100%);
    }

    .toggle-button {
      position: relative;
      z-index: 2;
      background: transparent;
      border: none;
      cursor: pointer;
      display: flex;
    }
</style>
