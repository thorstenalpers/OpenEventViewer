<script lang="ts">
	import ActivityIcon from '@lucide/svelte/icons/activity';
	import SparklesIcon from '@lucide/svelte/icons/sparkles';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import EventsTable from '$lib/components/events-table.svelte';
	import LevelBadge from '$lib/components/level-badge.svelte';
	import type { Incident } from '$lib/bridge/contract';
	import { i18n } from '$lib/i18n/index.svelte';
	import { cn } from '$lib/utils';
	import { createEventsTable } from '$lib/stores/events-table.svelte';
	import { DAY_CHOICES, diagnose } from '$lib/stores/diagnose.svelte';
	import { assistant } from '$lib/stores/assistant.svelte';

	const t = $derived(i18n.t);

	const data = createEventsTable(() => diagnose.bundle?.events ?? [], 'diagnose');

	function shown(value: string): string {
		const parsed = Date.parse(value);
		return Number.isNaN(parsed) ? value : new Date(parsed).toLocaleString();
	}

	async function send() {
		const bundle = diagnose.bundle;
		if (!bundle) return;
		assistant.attach({
			id: `bundle:${bundle.incident.id}`,
			kind: 'bundle',
			title: `${t.diagnose.kinds[bundle.incident.kind]} — ${shown(bundle.incident.time)}`,
			text: bundle.prompt,
			events: bundle.events
		});
		assistant.draft = t.diagnose.question;
		await goto(resolve('/assistant'));
	}
</script>

<div class="flex h-full flex-col gap-3 p-4 sm:p-6">
	<div class="flex flex-wrap items-center gap-2">
		<h1 class="flex items-center gap-2 text-xl font-semibold">
			<ActivityIcon class="size-5" />
			{t.diagnose.title}
		</h1>
		<div class="ms-auto flex items-center gap-1">
			{#each DAY_CHOICES as days (days)}
				<button
					type="button"
					aria-pressed={diagnose.days === days}
					onclick={() => (diagnose.days = days)}
					class={cn(
						'cursor-pointer rounded-md border px-2 py-1 text-xs transition-colors',
						diagnose.days === days
							? 'border-primary bg-primary/10 text-primary'
							: 'border-input text-muted-foreground hover:bg-accent/40'
					)}
				>
					{t.diagnose.days(days)}
				</button>
			{/each}
			<Button onclick={() => diagnose.load()} disabled={diagnose.scanning}>
				{diagnose.scanning ? t.diagnose.scanning : t.diagnose.scan}
			</Button>
		</div>
	</div>

	<p class="text-sm text-muted-foreground">{t.diagnose.subtitle}</p>

	{#if diagnose.error}
		<p class="text-sm whitespace-pre-wrap text-destructive">{diagnose.error}</p>
	{/if}

	<div class="flex min-h-0 flex-1 flex-col gap-3">
		{#if diagnose.incidents.length === 0}
			<p class="text-sm text-muted-foreground">
				{diagnose.scanning ? t.common.loading : t.diagnose.nothing}
			</p>
		{:else}
			<ul class="flex max-h-64 flex-col divide-y overflow-y-auto rounded-md border">
				{#each diagnose.incidents as incident (incident.id)}
					{@render row(incident)}
				{/each}
			</ul>
		{/if}

		{#if diagnose.opening}
			<p class="text-sm text-muted-foreground">{t.common.loading}</p>
		{/if}

		{#if diagnose.bundle}
			{@const bundle = diagnose.bundle}
			<div class="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
				<span>{t.diagnose.window(shown(bundle.from), shown(bundle.to))}</span>
				<span>·</span>
				<span>{t.diagnose.inWindow(bundle.events.length)}</span>
				<Button size="sm" class="ms-auto" onclick={send}>
					<SparklesIcon class="size-3.5" />
					{t.diagnose.send}
				</Button>
			</div>

			<EventsTable {data} class="min-h-40 flex-1" />

			<details class="rounded-md border">
				<summary class="cursor-pointer px-3 py-2 text-xs font-medium">
					{t.diagnose.previewBundle}
				</summary>
				<pre
					data-testid="bundle-preview"
					class="max-h-64 overflow-auto px-3 pb-3 text-[11px] whitespace-pre-wrap">{bundle.prompt}</pre>
			</details>
		{/if}
	</div>
</div>

{#snippet row(incident: Incident)}
	<li>
		<button
			type="button"
			onclick={() => diagnose.open(incident)}
			aria-pressed={diagnose.selectedId === incident.id}
			class={cn(
				'flex w-full cursor-pointer flex-wrap items-center gap-2 px-3 py-2 text-start text-sm hover:bg-muted/50',
				diagnose.selectedId === incident.id && 'bg-muted'
			)}
		>
			<LevelBadge level={incident.event.level} label={incident.event.levelName} />
			<Badge variant="accent">{t.diagnose.kinds[incident.kind]}</Badge>
			<span class="text-xs text-muted-foreground tabular-nums">{shown(incident.time)}</span>
			<span class="min-w-0 flex-1 truncate">{incident.headline}</span>
		</button>
	</li>
{/snippet}
