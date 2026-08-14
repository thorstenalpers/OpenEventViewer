<script lang="ts">
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { call } from '$lib/bridge/client';
	import type { Question } from '$lib/bridge/contract';
	import { library } from '$lib/stores/library.svelte';
	import { notes } from '$lib/stores/notes.svelte';
	import { i18n } from '$lib/i18n/index.svelte';
	import { resolve } from '$app/paths';
	import NotePanel from '$lib/components/note-panel.svelte';
	import QuestionFigure from '$lib/components/question-figure.svelte';

	const t = $derived(i18n.t);

	let questions = $state<Question[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);

	const binder = $derived(library.selected);

	$effect(() => {
		const id = binder?.id;
		if (id === undefined) {
			questions = [];
			return;
		}
		loading = true;
		error = null;
		void notes.load(id);
		call('list_questions', { binderId: id, onlyReview: true })
			.then((result) => (questions = result))
			.catch((caught: unknown) => {
				error = caught instanceof Error ? caught.message : String(caught);
			})
			.finally(() => (loading = false));
	});
</script>

<div class="flex flex-col gap-3 p-4 sm:p-6">
	<header>
		<h1 class="text-xl font-semibold">{t.review.title}</h1>
		<p class="text-sm text-muted-foreground">
			{#if binder}
				{t.review.subtitle(questions.length, binder.title)}
			{:else}
				{t.common.noBinder}
			{/if}
		</p>
	</header>

	{#if error}
		<p class="text-sm text-destructive">{error}</p>
	{/if}

	{#if loading}
		<p class="text-sm text-muted-foreground">{t.common.loading}</p>
	{:else if binder && questions.length === 0}
		<Card>
			<CardContent class="py-10 text-center text-sm text-muted-foreground">
				{t.review.clean}
			</CardContent>
		</Card>
	{/if}

	{#each questions as question (question.id)}
		<Card>
			<CardHeader>
				<CardDescription class="flex flex-wrap items-center gap-2">
					<Badge variant="accent">#{question.number}</Badge>
					<span>{t.review.page(question.sourcePage)}</span>
					<Badge variant="neutral">{t.review.confidence(question.confidence.toFixed(2))}</Badge>
					{#each question.warnings as warning (warning.code)}
						<Badge variant="destructive">
							{t.review.warnings[warning.code] ?? warning.code}
						</Badge>
					{/each}
				</CardDescription>
				<CardTitle class="text-base leading-relaxed whitespace-pre-line">
					{question.stem}
				</CardTitle>
			</CardHeader>
			<CardContent class="flex flex-col gap-3 text-sm">
				{#if question.figures.length}
					<QuestionFigure hashes={question.figures} />
				{/if}

				<ul class="flex flex-col gap-1">
					{#each question.options as option (option.letter)}
						<li class:font-medium={option.isCorrect}>
							<span class="font-mono">{option.letter}</span>
							{option.text}
							{#if option.isCorrect}<span aria-label="correct">✓</span>{/if}
						</li>
					{/each}
				</ul>

				{#if question.needsSource}
					<p class="rounded-md bg-warning/10 px-3 py-2">{t.review.needsSource}</p>
				{/if}

				{#if question.matrix.length}
					<div>
						<p class="mb-1 text-xs text-muted-foreground">{t.review.matrixKey}</p>
						<ul class="flex flex-wrap gap-2">
							{#each question.matrix as box (box.index)}
								<li><Badge variant="neutral">Box {box.index}: {box.value}</Badge></li>
							{/each}
						</ul>
					</div>
				{/if}

				{#if question.explanation}
					<p class="whitespace-pre-line text-muted-foreground">{question.explanation}</p>
				{/if}

				<NotePanel questionId={question.id} />
			</CardContent>
		</Card>
	{/each}

	{#if binder && questions.length > 0}
		<Button href={resolve('/train')} class="self-start">{t.review.backToTraining}</Button>
	{/if}
</div>
