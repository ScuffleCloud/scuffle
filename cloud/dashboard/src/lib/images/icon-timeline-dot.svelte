<script lang="ts">
    import type { TimelineDotVariant } from "$components/assets/types";

    type Props = {
        variant: TimelineDotVariant;
    };

    let { variant = "default" }: Props = $props();

    type DotStyles = {
        outer: string;
        outerOpacity: string;
        inner: string;
    };

    const DOT_VARIANTS: Record<TimelineDotVariant, DotStyles> = {
        "default": {
            outer: "#181716",
            outerOpacity: "0.12",
            inner: "#282625",
        },
        "live": {
            outer: "#FFD5D3",
            outerOpacity: "1",
            inner: "#8E0119",
        },
        "finished": {
            outer: "#282625",
            outerOpacity: "1",
            inner: "#F5EFEC",
        },
    };

    const currentVariant = DOT_VARIANTS[variant as TimelineDotVariant];
</script>

<svg
    xmlns="http://www.w3.org/2000/svg"
    width="12"
    height="16"
    viewBox="0 0 12 16"
    fill="none"
>
    <circle
        cx="6"
        cy="8"
        r="6"
        fill={currentVariant.outer}
        fill-opacity={currentVariant.outerOpacity}
    />
    <circle
        cx="6"
        cy="8"
        r="2"
        fill={currentVariant.inner}
    />
</svg>
