<script lang="ts">
	import FilterPopover from './filter-popover.svelte';
	import { isEmptyTimeRange, type TimeRange } from '$lib/events';
	import { i18n } from '$lib/i18n/index.svelte';

	interface Props {
		title: string;
		range: TimeRange;
		onChange: (range: TimeRange | undefined) => void;
	}

	let { title, range, onChange }: Props = $props();

	const t = $derived(i18n.t);

	function short(value: string): string {
		const at = Date.parse(value);
		return Number.isNaN(at) ? value : new Date(at).toLocaleString();
	}

	const label = $derived(
		isEmptyTimeRange(range)
			? title
			: range.from && range.to
				? `${short(range.from)} – ${short(range.to)}`
				: range.from
					? t.events.filters.after(short(range.from))
					: t.events.filters.before(short(range.to))
	);

	function set(edge: 'from' | 'to', value: string) {
		const next: TimeRange = { ...range, [edge]: value };
		onChange(isEmptyTimeRange(next) ? undefined : next);
	}
</script>

<FilterPopover {label} {title} active={!isEmptyTimeRange(range)} width={300}>
	<div class="flex flex-col gap-2">
		<label class="flex flex-col gap-1 text-xs text-muted-foreground">
			{t.events.from}
			<input
				type="datetime-local"
				value={range.from}
				onchange={(event) => set('from', event.currentTarget.value)}
				class="h-7 rounded border border-input bg-background px-1.5 text-xs focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none"
			/>
		</label>
		<label class="flex flex-col gap-1 text-xs text-muted-foreground">
			{t.events.to}
			<input
				type="datetime-local"
				value={range.to}
				onchange={(event) => set('to', event.currentTarget.value)}
				class="h-7 rounded border border-input bg-background px-1.5 text-xs focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none"
			/>
		</label>
		<p class="text-[11px] text-muted-foreground">{t.events.filters.timeHint}</p>
		{#if !isEmptyTimeRange(range)}
			<button
				type="button"
				class="cursor-pointer rounded px-1 py-0.5 text-start text-xs text-muted-foreground hover:bg-muted/60"
				onclick={() => onChange(undefined)}
			>
				{t.events.filters.clear}
			</button>
		{/if}
	</div>
</FilterPopover>
