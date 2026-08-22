<script lang="ts">
	import RefreshIcon from '@lucide/svelte/icons/refresh-cw';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Select } from '$lib/components/ui/select';
	import EventsTable from '$lib/components/events-table.svelte';
	import { i18n } from '$lib/i18n/index.svelte';
	import { cn } from '$lib/utils';
	import type { EventRecord } from '$lib/bridge/contract';
	import { ALL_CHANNELS, LEVELS, PINNED_CHANNELS, RANGES, keyOf, levelKey } from '$lib/events';
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

	const rangeOptions = $derived(
		RANGES.map((range) => ({ value: range, label: t.events.ranges[range] }))
	);

	let channel = $state(events.channel);
	let range = $state(events.range);

	$effect(() => {
		events.channel = channel;
	});
	$effect(() => {
		events.range = range;
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

		<div class="flex flex-col gap-1 text-xs text-muted-foreground">
			{t.events.level}
			<div class="flex h-9 items-center gap-1">
				{#each LEVELS as level (level)}
					<button
						type="button"
						aria-pressed={events.levels.has(level)}
						onclick={() => events.toggleLevel(level)}
						class={cn(
							'cursor-pointer rounded-md border px-2 py-1 text-xs transition-colors',
							events.levels.has(level)
								? 'border-primary bg-primary/10 text-primary'
								: 'border-input text-muted-foreground hover:bg-accent/40'
						)}
					>
						{t.events.levels[levelKey(level)]}
					</button>
				{/each}
			</div>
		</div>

		<label class="flex flex-col gap-1 text-xs text-muted-foreground">
			{t.events.range}
			<Select bind:value={range} options={rangeOptions} aria-label={t.events.range} class="w-40" />
		</label>

		{#if events.range === 'custom'}
			<label class="flex flex-col gap-1 text-xs text-muted-foreground">
				{t.events.from}
				<Input type="datetime-local" bind:value={events.from} class="w-52" />
			</label>
			<label class="flex flex-col gap-1 text-xs text-muted-foreground">
				{t.events.to}
				<Input type="datetime-local" bind:value={events.to} class="w-52" />
			</label>
		{/if}

		<label class="flex flex-col gap-1 text-xs text-muted-foreground">
			{t.events.eventIds}
			<Input bind:value={events.eventIdText} placeholder="41, 6008" class="w-32" />
		</label>

		<label class="flex flex-col gap-1 text-xs text-muted-foreground">
			{t.events.providers}
			<Input bind:value={events.providerText} placeholder={t.events.providersHint} class="w-56" />
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
			<Button size="sm" variant="ghost" onclick={() => data.clearColumnFilters()}>
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

	<EventsTable
		{data}
		class="min-h-0 flex-1"
		selectedId={events.selectedId}
		onSelect={(event: EventRecord) =>
			events.select(keyOf(event) === events.selectedId ? null : event)}
	/>
</div>
