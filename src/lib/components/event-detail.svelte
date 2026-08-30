<script lang="ts">
	import SearchIcon from '@lucide/svelte/icons/globe';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import XIcon from '@lucide/svelte/icons/x';
	import { Button } from '$lib/components/ui/button';
	import LevelBadge from '$lib/components/level-badge.svelte';
	import { call } from '$lib/bridge/client';
	import type { EventRecord } from '$lib/bridge/contract';
	import { i18n } from '$lib/i18n/index.svelte';
	import { cn } from '$lib/utils';

	interface Props {
		event: EventRecord;
		onSearch: (event: EventRecord) => void;
		onClose: () => void;
	}

	let { event, onSearch, onClose }: Props = $props();

	const t = $derived(i18n.t);

	type Tab = 'general' | 'data' | 'xml';
	let tab = $state<Tab>('general');
	let xml = $state<string | null>(null);
	let xmlError = $state<string | null>(null);
	let copied = $state(false);

	// The XML is a second query per event, so it is fetched when the tab is opened rather than with
	// the row. A different event invalidates whatever was fetched for the last one.
	$effect(() => {
		const key = `${event.channel}:${event.recordId}`;
		void key;
		xml = null;
		xmlError = null;
		tab = 'general';
		copied = false;
	});

	$effect(() => {
		if (tab !== 'xml' || xml !== null) return;
		const { channel, recordId } = event;
		call('events_xml', { channel, recordId })
			.then((text) => (xml = text))
			.catch((error: unknown) => {
				xmlError = error instanceof Error ? error.message : String(error);
			});
	});

	const tabs = $derived<{ id: Tab; label: string }[]>([
		{ id: 'general', label: t.detail.general },
		{ id: 'data', label: t.detail.data },
		{ id: 'xml', label: t.detail.xml }
	]);

	async function copy() {
		await navigator.clipboard.writeText(
			tab === 'xml' && xml ? xml : `${event.provider} ${event.eventId}\n${event.message}`
		);
		copied = true;
	}
</script>

<div class="flex max-h-72 flex-col rounded-md border">
	<div class="flex flex-wrap items-center gap-2 border-b px-3 py-2">
		<LevelBadge level={event.level} label={event.levelName} />
		<span class="text-sm font-medium">{event.provider}</span>
		<span class="text-sm text-muted-foreground tabular-nums">{event.eventId}</span>
		<span class="text-xs text-muted-foreground">{event.timeCreated}</span>

		<div class="ms-auto flex items-center gap-1">
			<Button size="sm" variant="outline" onclick={() => onSearch(event)}>
				<SearchIcon class="size-3.5" />
				{t.detail.search}
			</Button>
			<Button size="sm" variant="ghost" onclick={copy}>
				<CopyIcon class="size-3.5" />
				{copied ? t.detail.copied : t.detail.copy}
			</Button>
			<Button size="sm" variant="ghost" aria-label={t.detail.close} onclick={onClose}>
				<XIcon class="size-3.5" />
			</Button>
		</div>
	</div>

	<div class="flex gap-1 border-b px-3 py-1.5">
		{#each tabs as entry (entry.id)}
			<button
				type="button"
				aria-pressed={tab === entry.id}
				onclick={() => (tab = entry.id)}
				class={cn(
					'cursor-pointer rounded px-2 py-0.5 text-xs transition-colors',
					tab === entry.id ? 'bg-muted font-medium' : 'text-muted-foreground hover:bg-muted/50'
				)}
			>
				{entry.label}
			</button>
		{/each}
	</div>

	<div class="min-h-0 flex-1 overflow-auto p-3 text-sm">
		{#if tab === 'general'}
			<p class="whitespace-pre-wrap">{event.message}</p>
			<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 pt-3 text-xs">
				<dt class="text-muted-foreground">{t.events.columns.channel}</dt>
				<dd>{event.channel}</dd>
				<dt class="text-muted-foreground">{t.events.columns.task}</dt>
				<dd>{event.task}</dd>
				<dt class="text-muted-foreground">{t.events.columns.computer}</dt>
				<dd>{event.computer}</dd>
				<dt class="text-muted-foreground">{t.detail.recordId}</dt>
				<dd class="tabular-nums">{event.recordId}</dd>
				{#if event.keywords.length}
					<dt class="text-muted-foreground">{t.detail.keywords}</dt>
					<dd>{event.keywords.join(', ')}</dd>
				{/if}
			</dl>
		{:else if tab === 'data'}
			{#if event.eventData.length}
				<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-xs">
					{#each event.eventData as item (item.name)}
						<dt class="text-muted-foreground">{item.name}</dt>
						<dd class="break-all">{item.value}</dd>
					{/each}
				</dl>
			{:else}
				<p class="text-xs text-muted-foreground">{t.detail.noData}</p>
			{/if}
		{:else if xmlError}
			<p class="text-xs text-destructive">{xmlError}</p>
		{:else if xml === null}
			<p class="text-xs text-muted-foreground">{t.common.loading}</p>
		{:else}
			<pre class="text-[11px] whitespace-pre-wrap">{xml}</pre>
		{/if}
	</div>
</div>
