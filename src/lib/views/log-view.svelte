<script lang="ts">
	import TrashIcon from '@lucide/svelte/icons/trash-2';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Select } from '$lib/components/ui/select';
	import { Badge } from '$lib/components/ui/badge';
	import { cn } from '$lib/utils';
	import { log } from '$lib/stores/log.svelte';
	import type { LogLevel } from '$lib/bridge/contract';
	import { i18n } from '$lib/i18n/index.svelte';

	const t = $derived(i18n.t);

	$effect(() => {
		void log.refresh();
	});

	const levelOptions = $derived([
		{ value: 'all', label: t.log.levels.all },
		{ value: 'error', label: t.log.levels.error },
		{ value: 'warning', label: t.log.levels.warning },
		{ value: 'info', label: t.log.levels.info },
		{ value: 'debug', label: t.log.levels.debug }
	]);

	let level = $state<string>('all');
	$effect(() => {
		log.levelFilter = level as LogLevel | 'all';
	});

	const shown = $derived(
		log.entries.filter(
			(entry) =>
				(log.levelFilter === 'all' || entry.level === log.levelFilter) &&
				(log.messageFilter === '' ||
					`${entry.source} ${entry.message}`
						.toLowerCase()
						.includes(log.messageFilter.toLowerCase()))
		)
	);

	// Newest last, like a terminal. The buffer arrives in arrival order and stays that way.
	const LEVEL_CLASS: Record<string, string> = {
		error: 'text-destructive',
		warning: 'text-warning',
		info: 'text-foreground',
		debug: 'text-muted-foreground'
	};
</script>

<div class="flex h-full flex-col gap-3 p-4 sm:p-6">
	<header>
		<h1 class="text-xl font-semibold">{t.log.title}</h1>
		<p class="text-sm text-muted-foreground">{t.log.subtitle}</p>
	</header>

	<div class="flex flex-wrap items-center gap-2">
		<Input placeholder={t.log.filter} bind:value={log.messageFilter} class="max-w-sm" />
		<Select bind:value={level} options={levelOptions} aria-label={t.log.level} class="w-40" />
		<Button variant="ghost" onclick={() => log.clear()} aria-label={t.log.clear}>
			<TrashIcon class="size-4" />
		</Button>
		<span class="text-xs text-muted-foreground"
			>{t.log.count(shown.length, log.entries.length)}</span
		>
	</div>

	{#if log.error}
		<p class="text-sm text-destructive">{log.error}</p>
	{/if}

	<div class="min-h-0 flex-1 overflow-auto rounded-md border">
		{#if shown.length === 0}
			<p class="p-4 text-sm text-muted-foreground">
				{log.loading ? t.common.loading : t.log.empty}
			</p>
		{:else}
			<ul class="divide-y font-mono text-xs">
				{#each shown as entry, index (entry.timestamp + index)}
					<li class="flex gap-3 px-3 py-1.5">
						<span class="shrink-0 text-muted-foreground tabular-nums">{entry.timestamp}</span>
						<Badge variant={entry.level === 'error' ? 'destructive' : 'neutral'} class="shrink-0">
							{entry.source}
						</Badge>
						<span class={cn('flex-1 break-words whitespace-pre-wrap', LEVEL_CLASS[entry.level])}>
							{entry.message}
						</span>
					</li>
				{/each}
			</ul>
		{/if}
	</div>
</div>
