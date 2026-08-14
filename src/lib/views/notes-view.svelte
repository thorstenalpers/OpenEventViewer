<script lang="ts">
	import SparklesIcon from '@lucide/svelte/icons/sparkles';
	import HeadphonesIcon from '@lucide/svelte/icons/headphones';
	import TrashIcon from '@lucide/svelte/icons/trash-2';
	import FileTextIcon from '@lucide/svelte/icons/file-text';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { call } from '$lib/bridge/client';
	import type { Artefact, Note } from '$lib/bridge/contract';
	import { library } from '$lib/stores/library.svelte';
	import { settings } from '$lib/stores/settings.svelte';
	import { viewState } from '$lib/stores/view-state.svelte';
	import { voice } from '$lib/stores/voice.svelte';
	import { i18n } from '$lib/i18n/index.svelte';
	import NoBinder from '$lib/components/no-binder.svelte';

	const t = $derived(i18n.t);

	const binder = $derived(library.selected);

	let notes = $state<Note[]>([]);
	let artefacts = $state<Artefact[]>([]);
	let draft = $state('');
	let busy = $state<'' | 'summary' | 'podcast'>('');
	let error = $state<string | null>(null);

	$effect(() => {
		const id = binder?.id;
		if (id === undefined) {
			notes = [];
			artefacts = [];
			return;
		}
		void Promise.all([
			call('list_notes', { binderId: id }),
			call('list_artefacts', { binderId: id })
		])
			.then(([stored, made]) => {
				notes = stored;
				artefacts = made;
			})
			.catch((caught: unknown) => {
				error = caught instanceof Error ? caught.message : String(caught);
			});
	});

	/** Notes kept for the whole exam rather than for a single question. */
	const own = $derived(notes.filter((note) => note.questionId === null));

	async function save() {
		const id = binder?.id;
		if (id === undefined || !draft.trim()) return;
		error = null;
		try {
			notes = await call('save_note', {
				binderId: id,
				note: { questionId: null, bodyMd: draft.trim() }
			});
			draft = '';
		} catch (caught) {
			error = caught instanceof Error ? caught.message : String(caught);
		}
	}

	async function summarise() {
		const id = binder?.id;
		if (id === undefined) return;
		error = null;
		busy = 'summary';
		try {
			artefacts = await call('notes_summarise', { binderId: id, source: settings.assistantSource });
		} catch (caught) {
			error = caught instanceof Error ? caught.message : String(caught);
		} finally {
			busy = '';
		}
	}

	/** Reads one summary out loud with whichever voice the settings page chose. */
	async function record(artefact: Artefact) {
		const id = binder?.id;
		if (id === undefined) return;
		error = null;
		busy = 'podcast';
		try {
			artefacts = await call('notes_podcast', {
				binderId: id,
				name: artefact.name,
				options: {
					includeAnswer: true,
					includeExplanation: true,
					pauseSeconds: viewState.podcast.pauseSeconds,
					format: viewState.podcast.format === 'wav' ? 'wav' : 'mp3',
					language: settings.locale === 'de' ? 'de' : 'en',
					voice: voice.chosen
				}
			});
		} catch (caught) {
			error = caught instanceof Error ? caught.message : String(caught);
		} finally {
			busy = '';
		}
	}

	async function remove(artefact: Artefact) {
		const id = binder?.id;
		if (id === undefined) return;
		artefacts = await call('delete_artefact', { binderId: id, name: artefact.name }).catch(
			() => artefacts
		);
	}

	function size(bytes: number): string {
		return bytes >= 1024 * 1024
			? `${(bytes / 1024 / 1024).toFixed(1)} MB`
			: `${Math.max(Math.round(bytes / 1024), 1)} KB`;
	}
</script>

<div class="flex flex-col gap-3 p-4 sm:p-6">
	<header>
		<h1 class="text-xl font-semibold">{t.notes.title}</h1>
		<p class="text-sm text-muted-foreground">
			{#if binder}{t.notes.subtitle(binder.title)}{/if}
		</p>
	</header>

	{#if !binder}
		<NoBinder />
	{:else}
		{#if error}
			<p class="text-sm text-destructive">{error}</p>
		{/if}

		<Card>
			<CardHeader>
				<CardTitle>{t.notes.ownTitle}</CardTitle>
				<CardDescription>{t.notes.ownBody}</CardDescription>
			</CardHeader>
			<CardContent class="flex flex-col gap-3">
				{#if own.length}
					<ul class="flex flex-col gap-2">
						{#each own as note (note.id)}
							<li class="rounded-md border px-3 py-2">
								<p class="text-sm whitespace-pre-wrap">{note.bodyMd}</p>
								<p class="mt-1 text-xs text-muted-foreground">{note.updatedAt}</p>
							</li>
						{/each}
					</ul>
				{:else}
					<p class="text-sm text-muted-foreground">{t.notes.empty}</p>
				{/if}

				<textarea
					bind:value={draft}
					rows="5"
					placeholder={t.notes.placeholder}
					class="w-full rounded-md border bg-background px-3 py-2 text-sm focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
				></textarea>
				<Button class="self-start" onclick={save} disabled={!draft.trim()}>{t.notes.save}</Button>
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>{t.notes.workshopTitle}</CardTitle>
				<CardDescription>{t.notes.workshopBody}</CardDescription>
			</CardHeader>
			<CardContent class="flex flex-col gap-3">
				<Button class="self-start" onclick={summarise} disabled={busy !== '' || own.length === 0}>
					<SparklesIcon class="size-4" />
					{busy === 'summary' ? t.notes.summarising : t.notes.summarise}
				</Button>

				{#if artefacts.length === 0}
					<p class="text-sm text-muted-foreground">{t.notes.noArtefacts}</p>
				{:else}
					<ul class="flex flex-col gap-2">
						{#each artefacts as artefact (artefact.name)}
							<li class="flex flex-wrap items-center gap-3 rounded-md border px-3 py-2 text-sm">
								{#if artefact.kind === 'md'}
									<FileTextIcon class="size-4 text-muted-foreground" />
								{:else}
									<HeadphonesIcon class="size-4 text-muted-foreground" />
								{/if}
								<span class="min-w-0 flex-1 truncate">{artefact.name}</span>
								<Badge variant="neutral">{size(artefact.bytes)}</Badge>
								{#if artefact.kind === 'md'}
									<Button
										size="sm"
										variant="outline"
										disabled={busy !== ''}
										onclick={() => record(artefact)}
									>
										<HeadphonesIcon class="size-4" />
										{busy === 'podcast' ? t.notes.recording : t.notes.asPodcast}
									</Button>
								{/if}
								<Button
									size="sm"
									variant="ghost"
									aria-label={t.notes.removeAria(artefact.name)}
									onclick={() => remove(artefact)}
								>
									<TrashIcon class="size-4" />
								</Button>
							</li>
						{/each}
					</ul>
				{/if}

				<p class="text-xs text-muted-foreground">{t.notes.assistantNote}</p>
			</CardContent>
		</Card>
	{/if}
</div>
