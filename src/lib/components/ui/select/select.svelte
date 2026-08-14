<script lang="ts">
	import type { HTMLSelectAttributes } from 'svelte/elements';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import { cn } from '$lib/utils';

	interface Option {
		value: string;
		label: string;
	}

	interface Props extends Omit<HTMLSelectAttributes, 'value' | 'class'> {
		value: string;
		options: Option[];
		class?: string;
	}

	// A native select rather than a listbox built from divs: it gets keyboard behaviour, screen
	// reader semantics and the platform's own popup for free, and none of that is worth rebuilding
	// for a settings pane.
	let { value = $bindable(), options, class: className, ...rest }: Props = $props();
</script>

<div class={cn('relative inline-flex items-center', className)}>
	<select
		bind:value
		class="h-9 w-full cursor-pointer appearance-none rounded-md border border-input bg-background py-2 ps-3 pe-8 text-sm transition-colors hover:bg-accent/40 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
		{...rest}
	>
		{#each options as option (option.value)}
			<option value={option.value}>{option.label}</option>
		{/each}
	</select>
	<ChevronDownIcon class="pointer-events-none absolute end-2 size-4 opacity-60" />
</div>
