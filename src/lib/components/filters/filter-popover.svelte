<script lang="ts">
	import type { Snippet } from 'svelte';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import { cn } from '$lib/utils';

	interface Props {
		/** What the closed button shows: the current selection, or the column name when there is none. */
		label: string;
		active: boolean;
		title: string;
		width?: number;
		children: Snippet;
	}

	let { label, active, title, width = 260, children }: Props = $props();

	let open = $state(false);
	let trigger = $state<HTMLButtonElement | null>(null);
	// Fixed rather than absolute: the header sits inside the table's own scroll container, and an
	// absolutely positioned panel would be clipped by it a few pixels below the button.
	let at = $state({ top: 0, left: 0 });

	function place() {
		const box = trigger?.getBoundingClientRect();
		if (!box) return;
		at = {
			top: box.bottom + 2,
			left: Math.min(box.left, window.innerWidth - width - 8)
		};
	}

	function toggle() {
		if (!open) place();
		open = !open;
	}

	function outside(event: MouseEvent) {
		if (!open) return;
		const target = event.target as Node | null;
		if (trigger?.contains(target ?? null)) return;
		if (panel?.contains(target ?? null)) return;
		open = false;
	}

	let panel = $state<HTMLElement | null>(null);
</script>

<svelte:window
	onmousedown={outside}
	onkeydown={(event: KeyboardEvent) => {
		if (event.key === 'Escape') open = false;
	}}
	onresize={() => (open = false)}
/>

<button
	bind:this={trigger}
	type="button"
	{title}
	aria-expanded={open}
	onclick={toggle}
	class={cn(
		'flex h-6 w-full cursor-pointer items-center gap-1 rounded border px-1.5 text-start text-xs font-normal',
		'focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none',
		active
			? 'border-primary/50 bg-primary/10 text-primary'
			: 'border-input bg-background text-muted-foreground hover:bg-accent/40'
	)}
>
	<span class="min-w-0 flex-1 truncate">{label}</span>
	<ChevronDownIcon class="size-3 shrink-0 opacity-60" />
</button>

{#if open}
	<div
		bind:this={panel}
		style:top="{at.top}px"
		style:left="{at.left}px"
		style:width="{width}px"
		class="fixed z-50 rounded-md border bg-background p-2 shadow-lg"
	>
		{@render children()}
	</div>
{/if}
