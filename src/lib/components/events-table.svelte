<script lang="ts">
	import ArrowUpIcon from '@lucide/svelte/icons/arrow-up';
	import ArrowDownIcon from '@lucide/svelte/icons/arrow-down';
	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
	import SparklesIcon from '@lucide/svelte/icons/sparkles';
	import LevelBadge from '$lib/components/level-badge.svelte';
	import { i18n } from '$lib/i18n/index.svelte';
	import { cn } from '$lib/utils';
	import type { EventRecord } from '$lib/bridge/contract';
	import { keyOf, type TimeRange } from '$lib/events';
	import { WIDTHS, type EventsTableData } from '$lib/stores/events-table.svelte';
	import ChoiceFilter from '$lib/components/filters/choice-filter.svelte';
	import TimeFilter from '$lib/components/filters/time-filter.svelte';
	import NumberFilter from '$lib/components/filters/number-filter.svelte';
	import type { Column } from '@tanstack/table-core';

	interface Props {
		data: EventsTableData;
		selectedId?: string | null;
		onSelect?: (event: EventRecord) => void;
		/** Absent on the diagnosis bundle, where the whole window is attached in one go. */
		onAsk?: (event: EventRecord) => void;
		class?: string;
	}

	let { data, selectedId = null, onSelect, onAsk, class: className }: Props = $props();

	const t = $derived(i18n.t);

	// Every row is the same height, which is what makes windowing a subtraction rather than a
	// measurement pass. A message longer than the cell is truncated, not wrapped, for that reason.
	const ROW_H = 32;
	const OVERSCAN = 10;

	let viewport = $state(0);
	let offset = $state(0);

	const rows = $derived(data.table.getRowModel().rows);
	const first = $derived(Math.max(0, Math.floor(offset / ROW_H) - OVERSCAN));
	const count = $derived(Math.ceil(viewport / ROW_H) + OVERSCAN * 2);
	const last = $derived(Math.min(rows.length, first + count));
	const visible = $derived(rows.slice(first, last));

	function scrolled(event: Event) {
		offset = (event.currentTarget as HTMLElement).scrollTop;
	}

	function label(column: string): string {
		return t.events.columns[column as keyof typeof t.events.columns] ?? column;
	}

	function shown(value: string): string {
		const parsed = Date.parse(value);
		return Number.isNaN(parsed) ? value : new Date(parsed).toLocaleString();
	}

	/** Every value the column holds before any filter narrowed it — see `choice-filter`. */
	function unfiltered(column: Column<EventRecord, unknown>): string[] {
		return data.table
			.getPreFilteredRowModel()
			.rows.map((row) => String(row.getValue<unknown>(column.id)));
	}

	const EMPTY_RANGE: TimeRange = { from: '', to: '' };
</script>

<div
	class={cn('relative overflow-auto rounded-md border', className)}
	bind:clientHeight={viewport}
	onscroll={scrolled}
>
	<table class="w-full table-fixed border-collapse text-sm">
		<colgroup>
			{#each WIDTHS as width, index (index)}
				<col style:width />
			{/each}
		</colgroup>
		<thead class="sticky top-0 z-10 bg-background">
			{#each data.table.getHeaderGroups() as headerGroup (headerGroup.id)}
				<tr class="border-b">
					{#each headerGroup.headers as header (header.id)}
						{@const sorted = header.column.getIsSorted()}
						{@const index = header.column.getSortIndex()}
						<th class="h-8 px-2 text-start font-medium text-muted-foreground select-none">
							<button
								type="button"
								class="flex w-full cursor-pointer items-center gap-1 truncate hover:text-foreground"
								onclick={header.column.getToggleSortingHandler()}
							>
								<span class="truncate">{label(header.column.id)}</span>
								{#if sorted === 'asc'}
									<ArrowUpIcon class="size-3 shrink-0" />
								{:else if sorted === 'desc'}
									<ArrowDownIcon class="size-3 shrink-0" />
								{:else}
									<ChevronsUpDownIcon class="size-3 shrink-0 opacity-40" />
								{/if}
								<!-- The rank only means something once a second column joins the sort. -->
								{#if sorted && index > -1 && data.table.getState().sorting.length > 1}
									<span class="text-[10px] tabular-nums">{index + 1}</span>
								{/if}
							</button>
						</th>
					{/each}
					<th class="h-8 px-2"></th>
				</tr>
				<tr class="border-b">
					{#each headerGroup.headers as header (header.id)}
						{@const column = header.column}
						{@const kind = column.columnDef.meta?.filter ?? 'text'}
						<th class="h-8 px-1 pb-1 font-normal">
							{#if kind === 'choice'}
								<ChoiceFilter
									title={label(column.id)}
									values={unfiltered(column)}
									selected={(column.getFilterValue() as string[]) ?? []}
									onChange={(selected: string[]) =>
										column.setFilterValue(selected.length ? selected : undefined)}
								/>
							{:else if kind === 'time'}
								<TimeFilter
									title={label(column.id)}
									range={(column.getFilterValue() as TimeRange) ?? EMPTY_RANGE}
									onChange={(range: TimeRange | undefined) => column.setFilterValue(range)}
								/>
							{:else if kind === 'number'}
								<NumberFilter
									title={label(column.id)}
									text={(column.getFilterValue() as string) ?? ''}
									onChange={(text: string | undefined) => column.setFilterValue(text)}
								/>
							{:else}
								<input
									type="text"
									value={(column.getFilterValue() as string) ?? ''}
									oninput={(event) => column.setFilterValue(event.currentTarget.value || undefined)}
									placeholder={label(column.id)}
									aria-label={`${label(column.id)} — ${t.events.columnFilter}`}
									class="h-6 w-full rounded border border-input bg-background px-1.5 text-xs font-normal focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none"
								/>
							{/if}
						</th>
					{/each}
					<th class="h-8 px-1 pb-1"></th>
				</tr>
			{/each}
		</thead>
		<tbody>
			{#if first > 0}
				<tr style:height="{first * ROW_H}px"></tr>
			{/if}
			{#each visible as row (row.id)}
				{@const event = row.original}
				{@const active = keyOf(event) === selectedId}
				<tr
					style:height="{ROW_H}px"
					class={cn(
						'cursor-pointer border-b last:border-0 hover:bg-muted/50',
						active && 'bg-muted'
					)}
					onclick={() => onSelect?.(event)}
				>
					<td class="px-2"><LevelBadge level={event.level} label={event.levelName} /></td>
					<td class="truncate px-2 tabular-nums">{shown(event.timeCreated)}</td>
					<td class="truncate px-2" title={event.provider}>{event.provider}</td>
					<td class="px-2 tabular-nums">{event.eventId}</td>
					<td class="truncate px-2" title={event.task}>{event.task}</td>
					<td class="truncate px-2" title={event.channel}>{event.channel}</td>
					<td class="truncate px-2">{event.computer}</td>
					<td class="truncate px-2 text-muted-foreground" title={event.message}>{event.message}</td>
					<td class="px-1 text-end">
						{#if onAsk}
							<button
								type="button"
								class="cursor-pointer rounded p-1 text-muted-foreground hover:bg-accent hover:text-primary"
								aria-label={t.events.ask}
								title={t.events.ask}
								onclick={(clicked) => {
									clicked.stopPropagation();
									onAsk?.(event);
								}}
							>
								<SparklesIcon class="size-3.5" />
							</button>
						{/if}
					</td>
				</tr>
			{/each}
			{#if last < rows.length}
				<tr style:height="{(rows.length - last) * ROW_H}px"></tr>
			{/if}
		</tbody>
	</table>

	{#if rows.length === 0}
		<p class="p-4 text-sm text-muted-foreground">{t.events.empty}</p>
	{/if}
</div>
