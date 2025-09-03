<script lang="ts">
    type Props = {
        leftLabel: string;
        rightLabel: string;
        value: "left" | "right";
        onToggle: (side: "left" | "right") => void;
    };

    let {
        leftLabel = "Option 1",
        rightLabel = "Option 2",
        value = "left",
        onToggle = () => {},
    }: Props = $props();

    function handleClick(side: "left" | "right") {
        value = side;
        onToggle(side);
    }
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
            onclick={() => handleClick("left")}
            type="button"
        >
            {leftLabel}
        </button>
        <button
            class="toggle-button"
            class:active={value === "right"}
            onclick={() => handleClick("right")}
            type="button"
        >
            {rightLabel}
        </button>
    </div>
</div>

<style>
    .toggle-container {
      display: inline-block;
    }

    .toggle-track {
      position: relative;
      display: flex;
      background: #f1f5f9;
      border-radius: 8px;
      padding: 4px;
      border: 1px solid #e2e8f0;
    }

    .slider-bg {
      position: absolute;
      top: 4px;
      left: 4px;
      width: calc(50% - 4px);
      height: calc(100% - 8px);
      background: white;
      border-radius: 6px;
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
      padding: 8px 16px;
      cursor: pointer;
      font-size: 14px;
      font-weight: 500;
      color: #64748b;
      transition: color 0.2s ease-in-out;
      border-radius: 6px;
      min-width: 80px;
    }

    .toggle-button:hover {
      color: #334155;
    }

    .toggle-button.active {
      color: #1e293b;
    }

    .toggle-button:focus {
      outline: none;
    }
</style>
