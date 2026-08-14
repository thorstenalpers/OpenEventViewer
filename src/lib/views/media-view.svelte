<script lang="ts">
	import TrashIcon from '@lucide/svelte/icons/trash-2';
	import PlayIcon from '@lucide/svelte/icons/play';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import { Select } from '$lib/components/ui/select';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { call } from '$lib/bridge/client';
	import type { Question, Video } from '$lib/bridge/contract';
	import { library } from '$lib/stores/library.svelte';
	import { viewState } from '$lib/stores/view-state.svelte';
	import { voice } from '$lib/stores/voice.svelte';
	import { i18n } from '$lib/i18n/index.svelte';

	const t = $derived(i18n.t);

	const formatOptions = $derived([
		{ value: 'mp3', label: t.media.formatMp3 },
		{ value: 'wav', label: t.media.formatWav }
	]);

	const languageOptions = $derived([
		{ value: 'en', label: t.media.spokenEn },
		{ value: 'de', label: t.media.spokenDe }
	]);

	let videos = $state<Video[]>([]);
	let questions = $state<Question[]>([]);
	let error = $state<string | null>(null);

	// Both in stores: a half-typed video and a synthesised episode are work, and clicking away
	// to look up a timestamp should not throw either away.
	// Not named `video`: the list below binds a loop variable of that name, and a store shadowed
	// by a loop item is a bug waiting for someone to move a line.
	const form = viewState.video;
	const podcast = viewState.podcast;
	let busy = $state(false);

	const binder = $derived(library.selected);

	$effect(() => {
		const id = binder?.id;
		if (id === undefined) {
			videos = [];
			questions = [];
			return;
		}
		void Promise.all([
			call('list_videos', { binderId: id }),
			call('list_questions', { binderId: id })
		])
			.then(([v, q]) => {
				videos = v;
				questions = q;
			})
			.catch((caught: unknown) => {
				error = caught instanceof Error ? caught.message : String(caught);
			});
	});

	/** `1:45`, `105` and `1:45:00` all mean the same thing to someone copying a YouTube timestamp. */
	function toSeconds(value: string): number {
		const parts = value.trim().split(':').filter(Boolean).map(Number);
		if (!parts.length || parts.some(Number.isNaN)) return 0;
		return parts.reduce((total, part) => total * 60 + part, 0);
	}

	function formatSeconds(total: number): string {
		const minutes = Math.floor(total / 60);
		const seconds = total % 60;
		return `${minutes}:${seconds.toString().padStart(2, '0')}`;
	}

	function formatMs(total: number): string {
		return formatSeconds(Math.round(total / 1000));
	}

	/** YouTube ignores a start time unless it is in the URL, so the anchor is baked into the link. */
	function withStart(video: Video): string {
		if (!video.startSeconds) return form.url;
		const separator = form.url.includes('?') ? '&' : '?';
		return form.url.includes('youtu')
			? `${form.url}${separator}t=${video.startSeconds}`
			: `${form.url}#t=${video.startSeconds}`;
	}

	async function add(event: SubmitEvent) {
		event.preventDefault();
		const id = binder?.id;
		if (id === undefined || !form.url.trim()) return;
		error = null;
		try {
			videos = await call('add_video', {
				binderId: id,
				video: {
					questionId: form.anchoredTo,
					url: form.url.trim(),
					title: form.title.trim() || form.url.trim(),
					startSeconds: toSeconds(form.startAt)
				}
			});
			form.title = '';
			form.url = '';
			form.startAt = '';
			form.anchoredTo = null;
		} catch (caught) {
			error = caught instanceof Error ? caught.message : String(caught);
		}
	}

	async function remove(video: Video) {
		const id = binder?.id;
		if (id === undefined) return;
		videos = await call('delete_video', { binderId: id, videoId: video.id });
	}

	async function record() {
		const id = binder?.id;
		if (id === undefined) return;
		busy = true;
		error = null;
		podcast.episode = null;
		try {
			podcast.episode = await call('podcast_build', {
				binderId: id,
				questionIds: [],
				options: {
					includeAnswer: podcast.includeAnswer,
					includeExplanation: podcast.includeExplanation,
					pauseSeconds: podcast.pauseSeconds,
					format: podcast.format === 'wav' ? 'wav' : 'mp3',
					language: podcast.language === 'de' ? 'de' : 'en',
					voice: voice.chosen
				}
			});
		} catch (caught) {
			error = caught instanceof Error ? caught.message : String(caught);
		} finally {
			busy = false;
		}
	}
</script>

<div class="flex flex-col gap-3 p-4 sm:p-6">
	<header>
		<h1 class="text-xl font-semibold">{t.media.title}</h1>
		<p class="text-sm text-muted-foreground">
			{#if binder}
				{t.media.subtitle(binder.title)}
			{:else}
				{t.common.noBinder}
			{/if}
		</p>
	</header>

	{#if error}
		<p class="text-sm text-destructive">{error}</p>
	{/if}

	<Card>
		<CardHeader>
			<CardTitle>{t.media.videosTitle}</CardTitle>
			<CardDescription>
				{t.media.videosBody}
			</CardDescription>
		</CardHeader>
		<CardContent class="flex flex-col gap-3">
			<form
				class="grid grid-cols-1 items-end gap-2 sm:grid-cols-[1fr_2fr_6rem_auto]"
				onsubmit={add}
			>
				<label class="flex flex-col gap-1 text-xs text-muted-foreground">
					{t.media.colTitle}
					<Input bind:value={form.title} placeholder="Clustering explained" />
				</label>
				<label class="flex flex-col gap-1 text-xs text-muted-foreground">
					{t.media.colUrl}
					<Input bind:value={form.url} placeholder="https://www.youtube.com/watch?v=…" required />
				</label>
				<label class="flex flex-col gap-1 text-xs text-muted-foreground">
					{t.media.colStart}
					<Input bind:value={form.startAt} placeholder="1:45" />
				</label>
				<Button type="submit">{t.media.add}</Button>
			</form>

			<label class="flex items-center gap-2 text-xs text-muted-foreground">
				{t.media.anchorTo}
				<select
					class="h-8 rounded-md border bg-background px-2 text-sm"
					bind:value={form.anchoredTo}
				>
					<option value={null}>{t.media.wholeBinder}</option>
					{#each questions as question (question.id)}
						<option value={question.id}>#{question.number}</option>
					{/each}
				</select>
			</label>

			{#if videos.length === 0}
				<p class="text-sm text-muted-foreground">{t.media.noVideos}</p>
			{:else}
				<ul class="flex flex-col gap-2">
					{#each videos as video (video.id)}
						<li class="flex items-center gap-3 rounded-md border px-3 py-2 text-sm">
							<span class="flex-1 truncate">{form.title}</span>
							{#if video.questionId}
								<Badge variant="accent">
									#{questions.find((q) => q.id === video.questionId)?.number ?? '?'}
								</Badge>
							{/if}
							{#if video.startSeconds}
								<Badge variant="neutral">{t.media.from(formatSeconds(video.startSeconds))}</Badge>
							{/if}
							<Button
								size="sm"
								variant="outline"
								href={withStart(video)}
								target="_blank"
								rel="external noreferrer"
							>
								<PlayIcon class="size-4" />
								{t.media.open}
							</Button>
							<Button
								size="sm"
								variant="ghost"
								aria-label={t.media.removeAria(form.title)}
								onclick={() => remove(video)}
							>
								<TrashIcon class="size-4" />
							</Button>
						</li>
					{/each}
				</ul>
			{/if}
		</CardContent>
	</Card>

	<Card>
		<CardHeader>
			<CardTitle>{t.media.podcastTitle}</CardTitle>
			<CardDescription>
				{t.media.podcastBody}
			</CardDescription>
		</CardHeader>
		<CardContent class="flex flex-col gap-3">
			<div class="flex flex-wrap items-center gap-6 text-sm">
				<label class="flex items-center gap-2">
					<input type="checkbox" bind:checked={podcast.includeAnswer} />
					{t.media.readAnswer}
				</label>
				<label class="flex items-center gap-2">
					<input type="checkbox" bind:checked={podcast.includeExplanation} />
					{t.media.readExplanation}
				</label>
				<label class="flex items-center gap-2">
					{t.media.pause}
					<Input
						type="number"
						min="0"
						max="30"
						step="1"
						bind:value={podcast.pauseSeconds}
						class="h-8 w-20"
					/>
					{t.media.seconds}
				</label>
				<!--
					Spans, not labels. A <label> around a native <select> forwards the click to the
					control, which opens the popup and closes it again in the same gesture — the
					dropdown then needs a second click to stay open. The select carries its own
					`aria-label`, so nothing is lost by not wrapping it.
				-->
				<span class="flex items-center gap-2 select-none">
					{t.media.format}
					<Select
						bind:value={podcast.format}
						options={formatOptions}
						aria-label={t.media.format}
						class="w-56"
					/>
				</span>
				<span class="flex items-center gap-2 select-none">
					{t.media.spokenLanguage}
					<Select
						bind:value={podcast.language}
						options={languageOptions}
						aria-label={t.media.spokenLanguage}
						class="w-40"
					/>
				</span>
			</div>

			<p class="text-xs text-muted-foreground">{t.media.spokenLanguageBody}</p>

			{#if !podcast.includeAnswer && !podcast.includeExplanation}
				<p class="text-xs text-muted-foreground">
					{t.media.recallOnly}
				</p>
			{/if}

			<Button class="self-start" onclick={record} disabled={busy || !binder}>
				{busy ? t.media.recording : t.media.record}
			</Button>

			{#if podcast.episode}
				<div class="rounded-md border p-3 text-sm">
					<p class="font-medium">
						{t.media.episodeMeta(
							formatMs(podcast.episode.durationMs),
							podcast.episode.chapters.length
						)}
					</p>
					<p class="mt-1 text-xs break-all text-muted-foreground">{podcast.episode.path}</p>
					<ul class="mt-2 flex flex-wrap gap-2">
						{#each podcast.episode.chapters as chapter (chapter.questionNumber)}
							<li>
								<Badge variant="neutral">
									{formatMs(chapter.offsetMs)} · {chapter.title}
								</Badge>
							</li>
						{/each}
					</ul>
				</div>
			{/if}
		</CardContent>
	</Card>
</div>
