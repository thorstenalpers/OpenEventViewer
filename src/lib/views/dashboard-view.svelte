<script lang="ts">
	import { resolve } from '$app/paths';
	import TargetIcon from '@lucide/svelte/icons/target';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Progress } from '$lib/components/ui/progress';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { call } from '$lib/bridge/client';
	import type { DashboardSummary, ExamTimeline } from '$lib/bridge/contract';
	import { library } from '$lib/stores/library.svelte';
	import { i18n } from '$lib/i18n/index.svelte';
	import Timeline from '$lib/components/exam-timeline.svelte';

	const t = $derived(i18n.t);

	let summary = $state<DashboardSummary | null>(null);
	let exams = $state<ExamTimeline[]>([]);
	let error = $state<string | null>(null);

	$effect(() => {
		// Re-reads whenever the project list changes, so an import shows up here without a reload.
		void library.binders.length;
		call('dashboard', {})
			.then((result) => (summary = result))
			.catch((caught: unknown) => {
				error = caught instanceof Error ? caught.message : String(caught);
			});
		call('timeline', {})
			.then((result) => (exams = result))
			.catch(() => (exams = []));
	});

	const percent = $derived(
		summary?.accuracy === null || summary?.accuracy === undefined
			? null
			: Math.round(summary.accuracy * 100)
	);
</script>

<div class="flex flex-col gap-3 p-4 sm:p-6">
	<header>
		<h1 class="text-xl font-semibold">{t.dashboard.title}</h1>
		<p class="text-sm text-muted-foreground">{t.dashboard.subtitle}</p>
	</header>

	{#if error}
		<p class="text-sm text-destructive">{error}</p>
	{:else if !summary}
		<!-- Named rather than left blank: a page with a heading and nothing under it reads as broken,
		     and the host answering slowly looks exactly like the host not answering at all. -->
		<p class="text-sm text-muted-foreground">{t.common.loading}</p>
	{:else}
		<div class="grid grid-cols-2 gap-3 lg:grid-cols-4">
			{#each [{ label: t.dashboard.projects, value: summary.projectCount }, { label: t.dashboard.questions, value: summary.questionCount }, { label: t.dashboard.dueToday, value: summary.dueToday }, { label: t.dashboard.weak, value: summary.weakCount }] as tile (tile.label)}
				<Card>
					<CardContent class="py-3">
						<p class="text-xs text-muted-foreground">{tile.label}</p>
						<p class="text-2xl font-semibold">{tile.value}</p>
					</CardContent>
				</Card>
			{/each}
		</div>

		<Card>
			<CardHeader>
				<CardTitle>{t.dashboard.progress}</CardTitle>
				<CardDescription>
					{#if percent === null}
						{t.dashboard.nothingAnswered}
					{:else}
						{t.dashboard.answeredOf(summary.answeredCount, summary.questionCount)}
					{/if}
				</CardDescription>
			</CardHeader>
			<CardContent class="flex flex-col gap-3">
				{#if percent !== null}
					<Progress label={t.dashboard.accuracy} value={percent} />
					<p class="text-sm text-muted-foreground">{t.dashboard.accuracyValue(percent)}</p>
				{/if}
				<div class="flex flex-wrap gap-2">
					{#if summary.dueToday > 0}
						<Button href={resolve('/train')}>
							<TargetIcon class="size-4" />
							{t.dashboard.startDue(summary.dueToday)}
						</Button>
					{/if}
					{#if summary.weakCount > 0}
						<Button variant="outline" href={resolve('/train')}>
							{t.dashboard.startWeak(summary.weakCount)}
						</Button>
					{/if}
					{#if summary.projectCount === 0}
						<Button href={resolve('/projects')}>{t.dashboard.createFirst}</Button>
					{/if}
				</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>{t.timeline.title}</CardTitle>
				<CardDescription>{t.timeline.subtitle}</CardDescription>
			</CardHeader>
			<CardContent>
				<Timeline {exams} />
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>{t.dashboard.recent}</CardTitle>
			</CardHeader>
			<CardContent>
				{#if summary.recentSessions.length === 0}
					<p class="text-sm text-muted-foreground">{t.dashboard.noSessions}</p>
				{:else}
					<ul class="flex flex-col gap-2">
						{#each summary.recentSessions as session (session.sessionId)}
							<li class="flex flex-wrap items-center gap-3 rounded-md border px-3 py-2 text-sm">
								<Badge variant="neutral">{t.dashboard.modes[session.mode] ?? session.mode}</Badge>
								<span class="flex-1 truncate">{session.binderTitle}</span>
								<span class="tabular-nums">{session.correct} / {session.total}</span>
								<span class="text-xs text-muted-foreground">{session.finishedAt}</span>
							</li>
						{/each}
					</ul>
				{/if}
			</CardContent>
		</Card>
	{/if}
</div>
