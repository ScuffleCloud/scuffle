<script lang="ts">
import { Select } from "melt/builders";

const options = [
    "Bleach",
    "Dan da Dan",
    "Re: Zero",
    "Jujutsu Kaisen",
    "Attack on Titan",
    "Death Note",
] as const;
type Option = (typeof options)[number];

const select = new Select<Option>();
</script>

<label for={select.ids.trigger}>Anime</label>
<button {...select.trigger}>
	{select.value ?? "Select an anime"}
</button>

<div {...select.content}>
	{#each options as option}
		<div {...select.getOption(option)}>
			{option}
		</div>
	{/each}
</div>
