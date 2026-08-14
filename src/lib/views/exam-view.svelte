<script lang="ts">
	import { resolve } from '$app/paths';
	import ExternalIcon from '@lucide/svelte/icons/external-link';
	import TrashIcon from '@lucide/svelte/icons/trash-2';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import TodoList, { type Step } from '$lib/components/todo-list.svelte';
	import { call } from '$lib/bridge/client';
	import type { Certification } from '$lib/bridge/contract';
	import { library } from '$lib/stores/library.svelte';
	import { i18n } from '$lib/i18n/index.svelte';
	import NoBinder from '$lib/components/no-binder.svelte';

	const t = $derived(i18n.t);

	const binder = $derived(library.selected);

	let passed = $state<Certification[]>([]);
	let ticked = $state<string[]>([]);
	let error = $state<string | null>(null);

	let newDate = $state('');
	let newNote = $state('');

	$effect(() => {
		const id = binder?.id;
		if (id === undefined) {
			passed = [];
			ticked = [];
			return;
		}
		void Promise.all([
			call('list_certifications', { binderId: id }),
			call('list_progress', { binderId: id })
		])
			.then(([dates, steps]) => {
				passed = dates;
				ticked = steps;
			})
			.catch((caught: unknown) => {
				error = caught instanceof Error ? caught.message : String(caught);
			});
	});

	/**
	 * The checklist.
	 *
	 * Three of these the app can answer for itself — the project exists, questions have been
	 * answered, a pass date is on file — and asking someone to tick a box the app already knows the
	 * answer to is asking them to keep two records of one fact. The rest are theirs to say.
	 */
	const steps = $derived<Step[]>([
		{ id: 'create', label: t.exam.steps.create, derived: true, done: binder !== null },
		{ id: 'intro', label: t.exam.steps.intro, derived: false, done: ticked.includes('intro') },
		{ id: 'study', label: t.exam.steps.study, derived: false, done: ticked.includes('study') },
		{ id: 'notes', label: t.exam.steps.notes, derived: false, done: ticked.includes('notes') },
		{
			id: 'train',
			label: t.exam.steps.train,
			derived: true,
			done: (binder?.attemptCount ?? 0) > 0
		},
		{ id: 'pass', label: t.exam.steps.pass, derived: true, done: passed.length > 0 }
	]);

	async function toggle(step: string, done: boolean) {
		const id = binder?.id;
		if (id === undefined) return;
		ticked = await call('set_progress', { binderId: id, step, done }).catch(() => ticked);
	}

	async function addDate(event: SubmitEvent) {
		event.preventDefault();
		const id = binder?.id;
		if (id === undefined || !newDate) return;
		error = null;
		try {
			passed = await call('add_certification', {
				binderId: id,
				passedAt: newDate,
				note: newNote.trim()
			});
			newDate = '';
			newNote = '';
		} catch (caught) {
			error = caught instanceof Error ? caught.message : String(caught);
		}
	}

	async function removeDate(certification: Certification) {
		const id = binder?.id;
		if (id === undefined) return;
		passed = await call('delete_certification', {
			binderId: id,
			certificationId: certification.id
		}).catch(() => passed);
	}

	function date(value: string): string {
		return new Date(value).toLocaleDateString(i18n.locale);
	}
</script>

<div class="flex flex-col gap-3 p-4 sm:p-6">
	{#if !binder}
		<NoBinder />
	{:else}
		<header class="flex flex-wrap items-end justify-between gap-4">
			<div>
				<h1 class="flex items-center gap-2 text-xl font-semibold">
					<Badge variant="accent">{binder.certification}</Badge>
					{binder.title}
				</h1>
				<p class="text-sm text-muted-foreground">{t.exam.subtitle(date(binder.importedAt))}</p>
			</div>
			<div class="flex flex-wrap gap-2">
				{#if binder.docUrl}
					<Button variant="outline" href={binder.docUrl} target="_blank" rel="external noreferrer">
						<ExternalIcon class="size-4" />
						{t.exam.studyGuide}
					</Button>
				{/if}
				<Button href={resolve('/train')}>{t.projects.train}</Button>
			</div>
		</header>

		{#if error}
			<p class="text-sm text-destructive">{error}</p>
		{/if}

		<!-- The checklist stays on screen while the summary scrolls: it is the thing being worked
		     through, and a list that scrolls away is a list nobody ticks. -->
		<div class="grid gap-3 lg:grid-cols-[1fr_18rem]">
			<div class="flex flex-col gap-3">
				<div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
					{#each [{ label: t.import.questions, value: binder.questionCount }, { label: t.import.needReview, value: binder.needsReviewCount }, { label: t.projects.columns.attempts, value: binder.attemptCount }, { label: t.exam.passedCount, value: passed.length }] as tile (tile.label)}
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
						<CardTitle>{t.exam.certifications}</CardTitle>
						<CardDescription>{t.exam.certificationsBody}</CardDescription>
					</CardHeader>
					<CardContent class="flex flex-col gap-3">
						{#if passed.length === 0}
							<p class="text-sm text-muted-foreground">{t.exam.noCertifications}</p>
						{:else}
							<ul class="flex flex-col gap-2">
								{#each passed as entry (entry.id)}
									<li class="flex items-center gap-3 rounded-md border px-3 py-2 text-sm">
										<Badge variant="accent">{date(entry.passedAt)}</Badge>
										<span class="flex-1 truncate text-muted-foreground">{entry.note}</span>
										<Button
											size="sm"
											variant="ghost"
											aria-label={t.exam.removeDate(date(entry.passedAt))}
											onclick={() => removeDate(entry)}
										>
											<TrashIcon class="size-4" />
										</Button>
									</li>
								{/each}
							</ul>
						{/if}

						<form class="flex flex-wrap items-end gap-2" onsubmit={addDate}>
							<label class="flex flex-col gap-1 text-xs text-muted-foreground">
								{t.exam.passedOn}
								<Input type="date" bind:value={newDate} required class="w-44" />
							</label>
							<label class="flex flex-1 flex-col gap-1 text-xs text-muted-foreground">
								{t.exam.note}
								<Input bind:value={newNote} placeholder={t.exam.notePlaceholder} />
							</label>
							<Button type="submit">
								<PlusIcon class="size-4" />
								{t.exam.addDate}
							</Button>
						</form>
					</CardContent>
				</Card>
			</div>

			<Card class="h-fit lg:sticky lg:top-4">
				<CardContent class="py-4">
					<TodoList {steps} {toggle} />
				</CardContent>
			</Card>
		</div>
	{/if}
</div>
