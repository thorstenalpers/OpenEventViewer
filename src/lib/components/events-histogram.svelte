<script lang="ts">
	import type { EventRecord } from '$lib/bridge/contract';
	import { errorTally, histogram, type Bucket } from '$lib/events';
	import { i18n } from '$lib/i18n/index.svelte';
	import { cn } from '$lib/utils';

	interface Props {
		events: EventRecord[];
		/** Pins the axis to the time filter's window; an open end falls back to the events. */
		span?: { from?: number; to?: number };
		class?: string;
	}

	let { events, span = {}, class: className }: Props = $props();

	const t = $derived(i18n.t);

	const HEIGHT = 56;
	const AXIS = 14;

	let width = $state(0);
	// One bar per ~9 px, so the chart is denser on a wide window rather than stretched.
	const columns = $derived(Math.max(12, Math.min(160, Math.floor(width / 9) || 60)));
	const chart = $derived(histogram(events, columns, span));
	const hovered = $state<{ index: number | null; x: number; y: number }>({
		index: null,
		x: 0,
		y: 0
	});
	const bucket = $derived(hovered.index === null ? null : (chart.buckets[hovered.index] ?? null));

	/** Only the errors, because those are the ones a reader is hunting when they hover a red bar. */
	const tally = $derived(
		bucket ? errorTally(events, bucket.start, bucket.start + chart.sizeMs) : []
	);

	// Six rows and a remainder. A bucket in a busy hour can hold twenty distinct faults, and a card
	// that long stops being a glance.
	const TOP = 6;
	const rest = $derived(tally.slice(TOP).reduce((sum, entry) => sum + entry.count, 0));

	// Positioned off the bar rather than laid out under it: anything in the flow would change the
	// height of this box, and the table below it would jump on every hover.
	function enter(index: number, target: EventTarget | null) {
		const box = (target as HTMLElement | null)?.getBoundingClientRect();
		hovered.index = index;
		if (box) {
			hovered.x = box.left + box.width / 2;
			hovered.y = box.top;
		}
	}

	/**
	 * How tall one bar stands, and how much of it is red and amber.
	 *
	 * The bar is scaled by a square root — one spike of ten thousand would otherwise flatten every
	 * other bar to a single pixel, and the quiet stretches are the interesting part. The split
	 * inside it is then a straight proportion: scaling the segments separately would draw four
	 * errors out of ten as two thirds of the bar.
	 */
	function bar(entry: Bucket): { total: number; errors: number; warnings: number } {
		if (chart.peak === 0 || entry.total === 0) return { total: 0, errors: 0, warnings: 0 };
		const total = Math.max(
			2,
			Math.round((Math.sqrt(entry.total) / Math.sqrt(chart.peak)) * HEIGHT)
		);
		const errors =
			entry.errors === 0 ? 0 : Math.max(1, Math.round((entry.errors / entry.total) * total));
		const warnings =
			entry.warnings === 0
				? 0
				: Math.min(total - errors, Math.max(1, Math.round((entry.warnings / entry.total) * total)));
		return { total, errors, warnings };
	}

	function spanOf(entry: Bucket): string {
		const from = new Date(entry.start).toLocaleString();
		const to = new Date(entry.start + chart.sizeMs).toLocaleTimeString();
		return `${from} – ${to}`;
	}

	// Once the window crosses midnight, a bare "11:00 AM" at each end says nothing about which day
	// it belongs to — and every window longer than a few hours crosses one.
	const sameDay = $derived(
		new Date(chart.from).toDateString() === new Date(chart.to).toDateString()
	);

	function axis(at: number): string {
		const date = new Date(at);
		const time = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
		if (chart.sizeMs >= 24 * 60 * 60 * 1000) return date.toLocaleDateString();
		return sameDay ? time : `${date.toLocaleDateString()} ${time}`;
	}
</script>

<div
	class={cn('flex flex-col gap-1 rounded-md border px-3 py-2', className)}
	bind:clientWidth={width}
>
	<!-- One line, never two. The hover readout is the longest thing here, and letting it wrap would
	     grow the whole box and shove the table down every time the pointer crosses a bar. -->
	<div class="flex h-4 items-baseline gap-x-3 overflow-hidden">
		<span class="shrink-0 text-xs font-medium">{t.events.overTime}</span>
		{#if chart.buckets.length}
			<span class="shrink-0 text-[11px] text-muted-foreground">
				{t.events.bucketSize(chart.sizeMs / 60000)}
			</span>
		{/if}
		<span class="ms-auto min-w-0 truncate text-[11px] text-muted-foreground tabular-nums">
			{#if bucket}{spanOf(bucket)}{/if}
		</span>
	</div>

	{#if chart.buckets.length === 0}
		<!-- The same height the bars and their axis take, so an empty result does not resize the box. -->
		<p class="flex items-center text-xs text-muted-foreground" style:height="{HEIGHT + AXIS}px">
			{t.events.empty}
		</p>
	{:else}
		<!-- A row of divs rather than an SVG: fifty flex children with two coloured segments each is
		     less code than the same thing in path data, and it inherits the theme's own tokens. -->
		<div
			class="flex shrink-0 items-end gap-px"
			style:height="{HEIGHT}px"
			role="img"
			aria-label={t.events.overTime}
			onmouseleave={() => (hovered.index = null)}
		>
			{#each chart.buckets as entry, index (entry.start)}
				{@const size = bar(entry)}
				<div
					role="presentation"
					onmouseenter={(moved) => enter(index, moved.currentTarget)}
					class="flex h-full min-w-px flex-1 cursor-default flex-col justify-end"
					class:bg-muted={hovered.index === index}
				>
					<div
						class="w-full rounded-t-[1px] bg-primary/45"
						style:height="{size.total - size.errors - size.warnings}px"
					></div>
					<div class="w-full bg-warning" style:height="{size.warnings}px"></div>
					<div class="w-full bg-destructive" style:height="{size.errors}px"></div>
				</div>
			{/each}
		</div>

		<div
			class="flex shrink-0 items-end justify-between text-[10px] text-muted-foreground tabular-nums"
			style:height="{AXIS}px"
		>
			<span>{axis(chart.from)}</span>
			<span>{axis(chart.to)}</span>
		</div>
	{/if}
</div>

{#if bucket}
	<div
		style:left="{Math.min(
			Math.max(hovered.x, 130),
			(typeof window === 'undefined' ? 1280 : window.innerWidth) - 130
		)}px"
		style:top="{hovered.y}px"
		class="pointer-events-none fixed z-50 w-64 -translate-x-1/2 -translate-y-full rounded-md border bg-background p-2 text-xs shadow-lg"
	>
		<p class="pb-1 text-[11px] text-muted-foreground tabular-nums">{spanOf(bucket)}</p>
		<p class="font-medium">
			{t.events.bucketCount(bucket.total, bucket.errors, bucket.warnings)}
		</p>
		{#if tally.length}
			<ul class="flex flex-col gap-0.5 pt-1.5">
				{#each tally.slice(0, TOP) as entry (entry.label)}
					<li class="flex items-baseline gap-2">
						<span class="size-1.5 shrink-0 rounded-full bg-destructive"></span>
						<span class="min-w-0 flex-1 truncate" title={entry.label}>{entry.label}</span>
						<span class="shrink-0 text-muted-foreground tabular-nums">{entry.count}</span>
					</li>
				{/each}
				{#if rest > 0}
					<li class="ps-3.5 text-[11px] text-muted-foreground">
						{t.events.andMore(tally.length - TOP, rest)}
					</li>
				{/if}
			</ul>
		{/if}
	</div>
{/if}
