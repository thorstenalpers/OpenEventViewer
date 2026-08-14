<script lang="ts">
	import type { ColumnDef } from '@tanstack/table-core';
	import ArrowUpDownIcon from '@lucide/svelte/icons/arrow-up-down';
	import { Badge } from '$lib/components/ui/badge';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import {
		Table,
		TableBody,
		TableCell,
		TableHead,
		TableHeader,
		TableRow
	} from '$lib/components/ui/table';
	import { createDataTable } from '$lib/table.svelte';
	import { call } from '$lib/bridge/client';
	import type { QuestionStat } from '$lib/bridge/contract';
	import { library } from '$lib/stores/library.svelte';
	import { i18n } from '$lib/i18n/index.svelte';

	const t = $derived(i18n.t);

	let stats = $state<QuestionStat[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);

	const binder = $derived(library.selected);

	$effect(() => {
		const id = binder?.id;
		if (id === undefined) {
			stats = [];
			return;
		}
		loading = true;
		error = null;
		call('question_stats', { binderId: id })
			.then((result) => (stats = result))
			.catch((caught: unknown) => {
				error = caught instanceof Error ? caught.message : String(caught);
			})
			.finally(() => (loading = false));
	});

	interface TopicRow {
		topic: number | null;
		questions: number;
		attempts: number;
		correct: number;
		accuracy: number | null;
	}

	// Rolled up from the same rows the table below shows, so the two can never disagree.
	const topics = $derived.by<TopicRow[]>(() => {
		const buckets: Record<string, TopicRow> = {};
		for (const stat of stats) {
			const key = String(stat.topic);
			const row = (buckets[key] ??= {
				topic: stat.topic,
				questions: 0,
				attempts: 0,
				correct: 0,
				accuracy: null
			});
			row.questions += 1;
			row.attempts += stat.attempts;
			row.correct += stat.correct;
		}
		return (
			Object.values(buckets)
				.map((row) => ({
					...row,
					accuracy: row.attempts > 0 ? row.correct / row.attempts : null
				}))
				// Weakest first: the point of the view is what to work on next.
				.sort((a, b) => (a.accuracy ?? 2) - (b.accuracy ?? 2))
		);
	});

	const answered = $derived(stats.some((stat) => stat.attempts > 0));

	const columns: ColumnDef<QuestionStat, never>[] = [
		{ accessorKey: 'number', header: 'question' },
		{ accessorKey: 'attempts', header: 'attempts' },
		{ accessorKey: 'accuracy', header: 'accuracy' },
		{ accessorKey: 'averageMs', header: 'averageTime' },
		{ accessorKey: 'lapses', header: 'lapses' },
		{ accessorKey: 'dueAt', header: 'due' }
	];

	const data = createDataTable<QuestionStat>(() => stats, columns, [
		{ id: 'accuracy', desc: false }
	]);

	function percent(value: number | null): string {
		return value === null ? '—' : `${Math.round(value * 100)}%`;
	}

	function seconds(value: number | null): string {
		return value === null ? '—' : `${(value / 1000).toFixed(1)} s`;
	}

	function date(value: string | null): string {
		return value ? new Date(`${value}Z`).toLocaleDateString(i18n.locale) : '—';
	}
</script>

<div class="flex flex-col gap-3 p-4 sm:p-6">
	<header>
		<h1 class="text-xl font-semibold">{t.stats.title}</h1>
		<p class="text-sm text-muted-foreground">
			{#if binder}
				{t.stats.subtitle(binder.title)}
			{:else}
				{t.common.noBinder}
			{/if}
		</p>
	</header>

	{#if error}
		<p class="text-sm text-destructive">{error}</p>
	{:else if loading}
		<p class="text-sm text-muted-foreground">{t.common.loading}</p>
	{:else if binder && !answered}
		<Card>
			<CardContent class="py-10 text-center text-sm text-muted-foreground">
				{t.stats.empty}
			</CardContent>
		</Card>
	{/if}

	{#if binder && answered}
		<Card>
			<CardHeader>
				<CardTitle>{t.stats.byTopic}</CardTitle>
			</CardHeader>
			<CardContent>
				<ul class="flex flex-col gap-2">
					{#each topics as row (row.topic ?? 'none')}
						<li class="flex items-center gap-3 rounded-md border px-3 py-2 text-sm">
							<span class="w-40">
								{row.topic === null ? t.stats.noTopic : `${t.stats.topic} ${row.topic}`}
							</span>
							<span class="w-24 text-muted-foreground">{t.stats.questionCount(row.questions)}</span>
							<div class="h-2 flex-1 overflow-hidden rounded-full bg-muted">
								<div
									class="h-full rounded-full bg-primary"
									style="width: {(row.accuracy ?? 0) * 100}%"
								></div>
							</div>
							<span class="w-14 text-end tabular-nums">{percent(row.accuracy)}</span>
						</li>
					{/each}
				</ul>
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>{t.stats.byQuestion}</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="rounded-md border">
					<Table>
						<TableHeader>
							{#each data.table.getHeaderGroups() as headerGroup (headerGroup.id)}
								<TableRow>
									{#each headerGroup.headers as header (header.id)}
										<TableHead>
											<button
												type="button"
												class="flex cursor-pointer items-center gap-1 hover:text-foreground"
												onclick={header.column.getToggleSortingHandler()}
											>
												{t.stats[header.column.columnDef.header as keyof typeof t.stats]}
												<ArrowUpDownIcon class="size-3 opacity-50" />
											</button>
										</TableHead>
									{/each}
								</TableRow>
							{/each}
						</TableHeader>
						<TableBody>
							{#each data.table.getRowModel().rows as row (row.id)}
								{@const stat = row.original}
								<TableRow>
									<TableCell class="max-w-md">
										<span class="font-medium">#{stat.number}</span>
										<span class="ms-2 text-muted-foreground">
											{stat.stem.split('\n')[0]?.slice(0, 70)}
										</span>
										{#if stat.needsSource}
											<Badge variant="neutral">{t.stats.excluded}</Badge>
										{/if}
									</TableCell>
									<TableCell>{stat.attempts || '—'}</TableCell>
									<TableCell>
										{#if stat.accuracy === null}
											<span class="text-muted-foreground">{t.stats.neverAnswered}</span>
										{:else}
											<span class:text-destructive={stat.accuracy < 0.5}>
												{percent(stat.accuracy)}
											</span>
										{/if}
									</TableCell>
									<TableCell>{seconds(stat.averageMs)}</TableCell>
									<TableCell>{stat.lapses || '—'}</TableCell>
									<TableCell>{date(stat.dueAt)}</TableCell>
								</TableRow>
							{/each}
						</TableBody>
					</Table>
				</div>
			</CardContent>
		</Card>
	{/if}
</div>
