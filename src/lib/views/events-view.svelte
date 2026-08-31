<script lang="ts">
	import RefreshIcon from '@lucide/svelte/icons/refresh-cw';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Select } from '$lib/components/ui/select';
	import EventsTable from '$lib/components/events-table.svelte';
	import EventDetail from '$lib/components/event-detail.svelte';
	import EventsHistogram from '$lib/components/events-histogram.svelte';
	import { i18n } from '$lib/i18n/index.svelte';
	import { cn } from '$lib/utils';
	import type { EventRecord } from '$lib/bridge/contract';
	import { ALL_CHANNELS, PINNED_CHANNELS, keyOf, type TimeRange } from '$lib/events';
	import { call } from '$lib/bridge/client';
	import { events } from '$lib/stores/events.svelte';
	import { createEventsTable } from '$lib/stores/events-table.svelte';

	const t = $derived(i18n.t);

	const data = createEventsTable(() => events.events, 'events');

	// A thousand channels sorted alphabetically buries System under Microsoft-Windows-AAD.
	const channelOptions = $derived([
		{ value: ALL_CHANNELS, label: t.events.allChannels },
		...PINNED_CHANNELS.map((channel) => ({ value: channel, label: channel })),
		...events.channels
			.filter((channel) => !PINNED_CHANNELS.includes(channel))
			.map((channel) => ({ value: channel, label: channel }))
	]);

	// The same wall-clock reading `inTimeRange` uses, so the axis and the rows agree.
	function wallClock(value: string | undefined): number | undefined {
		if (!value) return undefined;
		const parsed = Date.parse(value);
		return Number.isNaN(parsed) ? undefined : parsed;
	}

	const timeSpan = $derived.by(() => {
		const range = data.table.getColumn('time')?.getFilterValue() as TimeRange | undefined;
		let from = wallClock(range?.from);
		// A truncated load holds no answer for anything older than its oldest row. Drawing the
		// filter's full window anyway would show quiet weeks that are really rows past the cap.
		if (from !== undefined && events.truncated) {
			const oldest = wallClock(events.events.at(-1)?.timeCreated);
			if (oldest !== undefined && oldest > from) from = oldest;
		}
		return { from, to: wallClock(range?.to) };
	});

	function firstLine(message: string): string {
		return message.split('\n')[0]?.trim() ?? '';
	}

	// The default browser rather than a webview of our own: a search result is the web, and the web
	// belongs where the user already has their bookmarks, their sign-ins and their ad blocker.
	async function search(event: EventRecord) {
		const terms = [event.provider, `Event ID ${event.eventId}`, firstLine(event.message)]
			.filter(Boolean)
			.join(' ');
		await call('open_url', {
			url: `https://www.google.com/search?q=${encodeURIComponent(terms)}`
		});
	}

	let channel = $state(events.channel);

	$effect(() => {
		events.channel = channel;
	});

	$effect(() => {
		void events.loadChannels();
		void events.load();
	});
</script>

<div class="flex h-full flex-col gap-3 p-4 sm:p-6">
	<div class="flex flex-wrap items-end gap-2">
		<label class="flex flex-col gap-1 text-xs text-muted-foreground">
			{t.events.channel}
			<Select
				bind:value={channel}
				options={channelOptions}
				aria-label={t.events.channel}
				class="w-64"
			/>
		</label>

		<Button onclick={() => events.load()} disabled={events.loading}>
			<RefreshIcon class={cn('size-4', events.loading && 'animate-spin')} />
			{t.events.load}
		</Button>
	</div>

	<div class="flex flex-wrap items-center gap-3">
		<Input
			placeholder={t.events.keyword}
			bind:value={data.globalFilter}
			aria-label={t.events.keyword}
			class="max-w-sm"
		/>
		{#if data.columnFilters.length}
			<Button size="sm" variant="outline" onclick={() => data.clearColumnFilters()}>
				{t.events.clearColumnFilters}
			</Button>
		{/if}
		<p class="text-xs text-muted-foreground">
			{t.events.loaded(data.table.getRowModel().rows.length, events.events.length)}
			· {t.events.elapsed(events.elapsedMs)}
			{#if events.truncated}
				· <span class="text-warning">{t.events.truncated}</span>
			{/if}
		</p>
	</div>

	{#if events.error}
		<div class="rounded-md border border-destructive/40 bg-destructive/5 px-3 py-2">
			<p class="text-sm whitespace-pre-wrap text-destructive">{events.error}</p>
			{#if events.accessDenied}
				<p class="pt-1 text-xs text-muted-foreground">{t.events.securityHint}</p>
			{/if}
		</div>
	{/if}

	<EventsHistogram
		events={data.table.getRowModel().rows.map((row) => row.original)}
		span={timeSpan}
	/>

	<EventsTable
		{data}
		class="min-h-0 flex-1"
		selectedId={events.selectedId}
		onSelect={(event: EventRecord) =>
			events.select(keyOf(event) === events.selectedId ? null : event)}
		onSearch={search}
	/>

	{#if events.selected}
		<EventDetail event={events.selected} onSearch={search} onClose={() => events.select(null)} />
	{/if}
</div>
