<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import XIcon from '@lucide/svelte/icons/x';
	import TimerIcon from '@lucide/svelte/icons/timer';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Progress } from '$lib/components/ui/progress';
	import AssistantPanel from '$lib/components/assistant-panel.svelte';
	import QuestionFigure from '$lib/components/question-figure.svelte';
	import { call } from '$lib/bridge/client';
	import type { ChallengeResult } from '$lib/bridge/contract';
	import { library } from '$lib/stores/library.svelte';
	import { trainer } from '$lib/stores/trainer.svelte';
	import { resolve } from '$app/paths';
	import { viewState } from '$lib/stores/view-state.svelte';
	import { i18n } from '$lib/i18n/index.svelte';

	const t = $derived(i18n.t);

	const binder = $derived(library.selected);
	const question = $derived(trainer.current);
	const progress = $derived(
		trainer.session ? (trainer.index / trainer.session.questions.length) * 100 : 0
	);

	// In a store: a seed typed in and then lost is a challenge that cannot be repeated.
	const challenge = viewState.challenge;
	let board = $state<ChallengeResult[]>([]);
	let now = $state(Date.now());

	$effect(() => {
		if (!trainer.deadlineAt) return;
		const handle = setInterval(() => (now = Date.now()), 500);
		return () => clearInterval(handle);
	});

	const remainingSeconds = $derived(
		trainer.deadlineAt ? Math.max(0, Math.round((trainer.deadlineAt - now) / 1000)) : null
	);

	// The clock is the only thing that can end a timed run early, so expiry finishes the session
	// rather than merely showing zero.
	$effect(() => {
		if (remainingSeconds === 0 && trainer.session) void trainer.finish();
	});

	$effect(() => {
		const id = binder?.id;
		if (id === undefined) {
			board = [];
			return;
		}
		call('challenge_results', { binderId: id, seed: challenge.seed })
			.then((result) => (board = result))
			.catch(() => (board = []));
	});

	function optionState(letter: string): 'correct' | 'wrong' | 'chosen' | 'idle' {
		if (!trainer.lastResult) return trainer.selected.includes(letter) ? 'chosen' : 'idle';
		if (trainer.lastResult.answerLetters.includes(letter)) return 'correct';
		return trainer.selected.includes(letter) ? 'wrong' : 'idle';
	}

	const OPTION_CLASSES = {
		correct: 'border-success bg-success/10',
		wrong: 'border-destructive bg-destructive/10',
		chosen: 'border-primary bg-primary/5',
		idle: 'border-border hover:bg-muted/50'
	} as const;

	function clock(seconds: number): string {
		const minutes = Math.floor(seconds / 60);
		return `${minutes}:${(seconds % 60).toString().padStart(2, '0')}`;
	}

	// A challenge result belongs on the board of the entry the binder was published as. Without a
	// published entry there is no board, so the offer is not made rather than made and refused.
	let posted = $state<string | null>(null);
	let postError = $state<string | null>(null);

	function post(sessionId: number, entryId: string) {
		postError = null;
		call('catalog_post_result', { entryId, sessionId })
			.then((rows) => {
				const mine = rows.findIndex((row) => row.mine);
				posted = t.train.postedAt(mine + 1, rows.length);
			})
			.catch((caught: unknown) => {
				postError = caught instanceof Error ? caught.message : String(caught);
			});
	}
</script>

<div class="flex flex-col gap-3 p-4 sm:p-6">
	{#if !binder}
		<Card>
			<CardHeader>
				<CardTitle>{t.train.noBinderTitle}</CardTitle>
				<CardDescription>{t.train.noBinderBody}</CardDescription>
			</CardHeader>
			<CardContent><Button href={resolve('/projects')}>{t.import.choose}</Button></CardContent>
		</Card>
	{:else if trainer.summary}
		{@const summary = trainer.summary}
		<Card>
			<CardHeader>
				<CardTitle>{t.train.score(summary.correct, summary.total)}</CardTitle>
				<CardDescription>
					{t.train.summaryMeta(
						Math.round(summary.elapsedMs / 1000),
						summary.wrongQuestionIds.length
					)}
				</CardDescription>
			</CardHeader>
			<CardContent class="flex flex-col gap-3">
				<Progress
					label={t.train.sessionScore}
					value={(summary.correct / Math.max(summary.total, 1)) * 100}
				/>
				<div class="flex flex-wrap gap-2">
					{#if summary.wrongQuestionIds.length > 0}
						<Button
							onclick={() => trainer.start(summary.binderId, 'focus', summary.sessionId)}
							disabled={trainer.busy}
						>
							{t.train.startFocus(summary.wrongQuestionIds.length)}
						</Button>
					{/if}
					{#if summary.mode === 'challenge' && binder.remoteId && !posted}
						{@const entryId = binder.remoteId}
						<Button variant="outline" onclick={() => post(summary.sessionId, entryId)}>
							{t.train.postResult}
						</Button>
					{/if}
					<Button variant="outline" onclick={() => trainer.reset()}>{t.common.back}</Button>
				</div>
				{#if summary.mode === 'challenge' && !binder.remoteId}
					<p class="text-sm text-muted-foreground">{t.train.publishToPost}</p>
				{/if}
				{#if posted}
					<p class="text-sm">{posted}</p>
				{/if}
				{#if postError}
					<p class="text-sm text-destructive">{postError}</p>
				{/if}
				{#if summary.wrongQuestionIds.length === 0}
					<p class="text-sm text-muted-foreground">{t.train.nothingMissed}</p>
				{/if}
			</CardContent>
		</Card>
	{:else if !trainer.session}
		<Card>
			<CardHeader>
				<CardTitle>{binder.title}</CardTitle>
				<CardDescription>
					{t.train.binderMeta(binder.questionCount, binder.needsSourceCount)}
				</CardDescription>
			</CardHeader>
			<CardContent class="flex flex-wrap gap-2">
				<Button onclick={() => trainer.start(binder.id, 'practice')} disabled={trainer.busy}>
					{t.train.practice}
				</Button>
				<Button
					variant="outline"
					onclick={() => trainer.start(binder.id, 'due')}
					disabled={trainer.busy}
				>
					{t.train.dueToday}
				</Button>
				<Button
					variant="outline"
					onclick={() => trainer.start(binder.id, 'weak')}
					disabled={trainer.busy}
				>
					{t.train.weak}
				</Button>
				<Button
					variant="outline"
					onclick={() =>
						trainer.start(binder.id, 'exam', undefined, {
							seed: null,
							questionCount: challenge.questionCount,
							timeLimitSeconds: challenge.minutes * 60
						})}
					disabled={trainer.busy}
				>
					{t.train.exam}
				</Button>
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>{t.train.challengeTitle}</CardTitle>
				<CardDescription>
					{t.train.challengeBody}
				</CardDescription>
			</CardHeader>
			<CardContent class="flex flex-col gap-3">
				<div class="flex flex-wrap items-end gap-3 text-xs text-muted-foreground">
					<label class="flex flex-col gap-1">
						{t.train.seed}
						<Input type="number" bind:value={challenge.seed} class="h-8 w-28" />
					</label>
					<label class="flex flex-col gap-1">
						{t.train.questions}
						<Input type="number" min="1" bind:value={challenge.questionCount} class="h-8 w-24" />
					</label>
					<label class="flex flex-col gap-1">
						{t.train.minutes}
						<Input type="number" min="1" bind:value={challenge.minutes} class="h-8 w-24" />
					</label>
					<Button
						onclick={() =>
							trainer.start(binder.id, 'challenge', undefined, {
								seed: challenge.seed,
								questionCount: challenge.questionCount,
								timeLimitSeconds: challenge.minutes * 60
							})}
						disabled={trainer.busy}
					>
						{t.train.takeChallenge}
					</Button>
				</div>

				{#if board.length}
					<ul class="flex flex-col gap-1 text-sm">
						{#each board as result, place (result.sessionId)}
							<li class="flex items-center gap-3 rounded-md border px-3 py-2">
								<Badge variant={place === 0 ? 'accent' : 'neutral'}>#{place + 1}</Badge>
								<span class="flex-1">{t.train.score(result.correct, result.total)}</span>
								<span class="text-muted-foreground">
									{clock(Math.round(result.elapsedMs / 1000))}
								</span>
								<span class="text-xs text-muted-foreground">{result.finishedAt}</span>
							</li>
						{/each}
					</ul>
				{:else}
					<p class="text-sm text-muted-foreground">{t.train.noRuns(challenge.seed)}</p>
				{/if}
			</CardContent>
		</Card>
	{:else if question}
		<div class="flex items-center gap-4">
			<Progress label={t.train.sessionProgress} value={progress} class="flex-1" />
			{#if remainingSeconds !== null}
				<span
					class="flex items-center gap-1 text-sm tabular-nums"
					class:text-destructive={remainingSeconds < 60}
				>
					<TimerIcon class="size-4" />
					{clock(remainingSeconds)}
				</span>
			{/if}
			<span class="text-sm text-muted-foreground">
				{trainer.index + 1} / {trainer.session.questions.length}
			</span>
		</div>

		<Card>
			<CardHeader>
				<CardDescription class="flex items-center gap-2">
					<Badge variant="accent">#{question.number}</Badge>
					{#if trainer.required > 1}
						<span>{t.train.chooseN(trainer.required)}</span>
					{/if}
					{#if !trainer.revealsAnswer}
						<Badge variant="neutral">{t.train.noFeedback}</Badge>
					{/if}
				</CardDescription>
				<CardTitle class="text-base leading-relaxed whitespace-pre-line">
					{question.stem}
				</CardTitle>
			</CardHeader>
			<CardContent class="flex flex-col gap-2">
				{#if question.figures.length}
					<QuestionFigure hashes={question.figures} />
				{/if}

				{#each question.options as option (option.letter)}
					{@const state = optionState(option.letter)}
					<button
						type="button"
						disabled={trainer.lastResult !== null}
						onclick={() => trainer.toggle(option.letter)}
						class={`flex items-start gap-3 rounded-md border px-4 py-3 text-start text-sm transition-colors ${OPTION_CLASSES[state]}`}
					>
						<span class="font-mono font-semibold">{option.letter}</span>
						<span class="flex-1">{option.text}</span>
						{#if state === 'correct'}
							<CheckIcon class="size-4" />
						{:else if state === 'wrong'}
							<XIcon class="size-4" />
						{/if}
					</button>
				{/each}
			</CardContent>
			<CardContent class="flex flex-col gap-3">
				{#if trainer.lastResult}
					<p class="text-sm font-medium">
						{trainer.lastResult.correct ? t.train.correct : t.train.notCorrect}
					</p>
					{#if question.explanation}
						<p class="text-sm whitespace-pre-line text-muted-foreground">{question.explanation}</p>
					{/if}
					{#each question.references as reference (reference)}
						<a
							href={reference}
							target="_blank"
							rel="external noreferrer"
							class="text-sm break-all text-primary underline-offset-2 hover:underline"
						>
							{reference}
						</a>
					{/each}
					<AssistantPanel {question} />
					<Button onclick={() => trainer.next()} disabled={trainer.busy}>
						{trainer.remaining > 1 ? t.train.next : t.train.finish}
					</Button>
				{:else}
					<Button
						onclick={() => trainer.submit()}
						disabled={trainer.selected.length === 0 || trainer.busy}
					>
						{trainer.revealsAnswer ? t.train.check : t.train.answerAndContinue}
					</Button>
				{/if}
			</CardContent>
		</Card>
	{/if}

	{#if trainer.error}
		<p class="text-sm text-destructive">{trainer.error}</p>
	{/if}
</div>
