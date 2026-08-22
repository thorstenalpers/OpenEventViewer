<script lang="ts">
	import type { EventRecord } from '$lib/bridge/contract';
	import { histogram, type Bucket } from '$lib/events';
	import { i18n } from '$lib/i18n/index.svelte';
	import { cn } from '$lib/utils';

	interface Props {
		events: EventRecord[];
		class?: string;
	}

	let { events, class: className }: Props = $props();

	const t = $derived(i18n.t);

	const HEIGHT = 56;

	let width = $state(0);
	// One bar per ~9 px, so the chart is denser on a wide window rather than stretched.
	const columns = $derived(Math.max(12, Math.min(160, Math.floor(width / 9) || 60)));
	const chart = $derived(histogram(events, columns));
	const hovered = $state<{ index: number | null }>({ index: null });
	const bucket = $derived(hovered.index === null ? null : (chart.buckets[hovered.index] ?? null));

	/**
	 * How tall one bar stands, and how much of it is red.
	 *
	 * The bar is scaled by a square root — one spike of ten thousand would otherwise flatten every
	 * other bar to a single pixel, and the quiet stretches are the interesting part. The split
	 * inside it is then a straight proportion: scaling the two segments separately would draw four
	 * errors out of ten as two thirds of the bar.
	 */
	function bar(entry: Bucket): { total: number; errors: number } {
		if (chart.peak === 0 || entry.total === 0) return { total: 0, errors: 0 };
		const total = Math.max(
			2,
			Math.round((Math.sqrt(entry.total) / Math.sqrt(chart.peak)) * HEIGHT)
		);
		const errors =
			entry.errors === 0 ? 0 : Math.max(1, Math.round((entry.errors / entry.total) * total));
		return { total, errors };
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
	<div class="flex flex-wrap items-baseline gap-x-3 gap-y-0.5">
		<span class="text-xs font-medium">{t.events.overTime}</span>
		{#if chart.buckets.length}
			<span class="text-[11px] text-muted-foreground">
				{t.events.bucketSize(chart.sizeMs / 60000)}
			</span>
		{/if}
		<span class="ms-auto min-h-4 text-[11px] text-muted-foreground tabular-nums">
			{#if bucket}
				{spanOf(bucket)} · {t.events.bucketCount(bucket.total, bucket.errors)}
			{/if}
		</span>
	</div>

	{#if chart.buckets.length === 0}
		<p class="py-3 text-xs text-muted-foreground">{t.events.empty}</p>
	{:else}
		<!-- A row of divs rather than an SVG: fifty flex children with two coloured segments each is
		     less code than the same thing in path data, and it inherits the theme's own tokens. -->
		<div
			class="flex items-end gap-px"
			style:height="{HEIGHT}px"
			role="img"
			aria-label={t.events.overTime}
			onmouseleave={() => (hovered.index = null)}
		>
			{#each chart.buckets as entry, index (entry.start)}
				{@const size = bar(entry)}
				<div
					role="presentation"
					onmouseenter={() => (hovered.index = index)}
					class="flex h-full min-w-px flex-1 cursor-default flex-col justify-end"
					class:bg-muted={hovered.index === index}
				>
					<div
						class="w-full rounded-t-[1px] bg-primary/45"
						style:height="{size.total - size.errors}px"
					></div>
					<div class="w-full bg-destructive" style:height="{size.errors}px"></div>
				</div>
			{/each}
		</div>

		<div class="flex justify-between text-[10px] text-muted-foreground tabular-nums">
			<span>{axis(chart.from)}</span>
			<span>{axis(chart.to)}</span>
		</div>
	{/if}
</div>
