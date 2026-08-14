<script lang="ts">
	import type { HTMLTextareaAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	interface Props extends Omit<HTMLTextareaAttributes, 'class' | 'value'> {
		value: string;
		class?: string;
	}

	let { value = $bindable(), class: className, ...rest }: Props = $props();
</script>

<textarea
	bind:value
	class={cn(
		'min-h-20 w-full rounded-md border border-input bg-background px-3 py-2 text-sm transition-colors placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50',
		className
	)}
	{...rest}
></textarea>
