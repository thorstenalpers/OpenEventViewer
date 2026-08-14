<script lang="ts">
	import UploadIcon from '@lucide/svelte/icons/upload';
	import DownloadIcon from '@lucide/svelte/icons/download';
	import TrashIcon from '@lucide/svelte/icons/trash-2';
	import StarIcon from '@lucide/svelte/icons/star';
	import TrophyIcon from '@lucide/svelte/icons/trophy';
	import RefreshIcon from '@lucide/svelte/icons/refresh-cw';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import { Select } from '$lib/components/ui/select';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { call } from '$lib/bridge/client';
	import type {
		CatalogEntry,
		CatalogRating,
		CatalogSort,
		Identity,
		LeaderboardRow,
		SyncReport,
		UploadPreview
	} from '$lib/bridge/contract';
	import { library } from '$lib/stores/library.svelte';
	import { i18n } from '$lib/i18n/index.svelte';

	const t = $derived(i18n.t);

	let publisher = $state<Identity | null>(null);
	let renaming = $state('');
	let entries = $state<CatalogEntry[]>([]);
	let search = $state('');
	let sort = $state<CatalogSort>('recent');
	let error = $state<string | null>(null);
	let notice = $state<string | null>(null);
	let busy = $state('');

	let chosen = $state<string>('');
	let preview = $state<UploadPreview | null>(null);

	let openEntry = $state<string | null>(null);
	let ratings = $state<CatalogRating[]>([]);
	let stars = $state('5');
	let comment = $state('');
	let seeds = $state<number[]>([]);
	let seed = $state<number | null>(null);
	let board = $state<LeaderboardRow[]>([]);
	let sync = $state<SyncReport | null>(null);

	// A project with no questions cannot be published, so it is not offered as a choice either.
	const publishable = $derived(library.binders.filter((binder) => binder.questionCount > 0));

	$effect(() => {
		void library.refresh();
		void call('catalog_identity', {})
			.then((found) => (publisher = found))
			.catch(report);
	});

	$effect(() => {
		const filter = { search, sort };
		void call('catalog_list', { filter })
			.then((found) => (entries = found))
			.catch(report);
	});

	$effect(() => {
		const first = publishable[0];
		if (!chosen && first) chosen = String(first.id);
	});

	function report(caught: unknown) {
		error = caught instanceof Error ? caught.message : String(caught);
	}

	async function guard(what: string, run: () => Promise<void>) {
		error = null;
		busy = what;
		try {
			await run();
		} catch (caught) {
			report(caught);
		} finally {
			busy = '';
		}
	}

	async function reload() {
		entries = await call('catalog_list', { filter: { search, sort } });
	}

	function size(bytes: number): string {
		return bytes >= 1024 * 1024
			? `${(bytes / 1024 / 1024).toFixed(1)} MB`
			: `${Math.max(Math.round(bytes / 1024), 1)} KB`;
	}

	function rename() {
		void guard('rename', async () => {
			publisher = await call('catalog_rename', { name: renaming });
			renaming = '';
			await reload();
		});
	}

	function review() {
		void guard('preview', async () => {
			preview = await call('catalog_preview', { binderId: Number(chosen) });
		});
	}

	function publish() {
		void guard('publish', async () => {
			const published = await call('catalog_publish', { binderId: Number(chosen) });
			preview = null;
			notice = t.catalog.published(published.title);
			await reload();
		});
	}

	function withdraw(entry: CatalogEntry) {
		void guard(entry.id, async () => {
			entries = await call('catalog_withdraw', { entryId: entry.id });
			if (openEntry === entry.id) openEntry = null;
		});
	}

	function take(entry: CatalogEntry) {
		void guard(entry.id, async () => {
			const imported = await call('catalog_import', { entryId: entry.id });
			notice = t.catalog.imported(imported.title);
			await library.refresh();
		});
	}

	function open(entry: CatalogEntry) {
		if (openEntry === entry.id) {
			openEntry = null;
			return;
		}
		openEntry = entry.id;
		void guard(entry.id, async () => {
			ratings = await call('catalog_ratings', { entryId: entry.id });
			const mine = ratings.find((rating) => rating.mine);
			stars = String(mine?.stars ?? 5);
			comment = mine?.comment ?? '';
			seeds = await call('catalog_seeds', { entryId: entry.id });
			seed = seeds[0] ?? null;
			board = seed === null ? [] : await call('catalog_leaderboard', { entryId: entry.id, seed });
		});
	}

	function rate(entry: CatalogEntry) {
		void guard(entry.id, async () => {
			ratings = await call('catalog_rate', { entryId: entry.id, stars: Number(stars), comment });
			await reload();
		});
	}

	function showBoard(entryId: string, chosenSeed: number) {
		seed = chosenSeed;
		void guard(entryId, async () => {
			board = await call('catalog_leaderboard', { entryId, seed: chosenSeed });
		});
	}

	function move(direction: 'push' | 'pull') {
		void guard(direction, async () => {
			sync = await call(direction === 'push' ? 'progress_push' : 'progress_pull', {});
		});
	}

	const sortOptions = $derived([
		{ value: 'recent', label: t.catalog.sortRecent },
		{ value: 'rating', label: t.catalog.sortRating },
		{ value: 'questions', label: t.catalog.sortQuestions },
		{ value: 'title', label: t.catalog.sortTitle }
	]);
</script>

<div class="flex flex-col gap-3 p-4 sm:p-6">
	<header>
		<h1 class="text-xl font-semibold">{t.catalog.title}</h1>
		<p class="text-sm text-muted-foreground">{t.catalog.subtitle}</p>
	</header>

	<p class="rounded-md border border-dashed px-3 py-2 text-xs text-muted-foreground">
		{t.catalog.localNote}
	</p>

	{#if error}
		<p class="text-sm text-destructive">{error}</p>
	{/if}
	{#if notice}
		<p class="text-sm text-muted-foreground">{notice}</p>
	{/if}

	<Card>
		<CardHeader>
			<CardTitle>{t.catalog.publishTitle}</CardTitle>
			<CardDescription>{t.catalog.publishBody}</CardDescription>
		</CardHeader>
		<CardContent class="flex flex-col gap-3">
			<div class="flex flex-wrap items-center gap-2 text-sm">
				<span class="text-muted-foreground">{t.catalog.publishedAs}</span>
				<Badge variant="neutral">{publisher?.name ?? '…'}</Badge>
				<Input
					class="w-56"
					bind:value={renaming}
					placeholder={t.catalog.namePlaceholder}
					aria-label={t.catalog.rename}
				/>
				<Button size="sm" variant="outline" disabled={!renaming.trim()} onclick={rename}>
					{t.catalog.rename}
				</Button>
			</div>

			<div class="flex flex-wrap items-end gap-2">
				<label class="flex flex-col gap-1 text-sm">
					<span class="text-muted-foreground">{t.catalog.project}</span>
					<Select
						class="w-72"
						bind:value={chosen}
						options={publishable.map((binder) => ({
							value: String(binder.id),
							label: `${binder.certification} — ${binder.title}`
						}))}
					/>
				</label>
				<Button variant="outline" disabled={!chosen || busy !== ''} onclick={review}>
					<UploadIcon class="size-4" />
					{t.catalog.review}
				</Button>
			</div>

			{#if preview}
				<div class="flex flex-col gap-2 rounded-md border px-3 py-2">
					<p class="text-sm font-medium">{t.catalog.previewTitle}</p>
					<p class="text-xs text-muted-foreground">{t.catalog.previewBody}</p>
					<ul class="flex flex-wrap gap-2 text-xs">
						<Badge variant="neutral">{t.catalog.questions(preview.questionCount)}</Badge>
						<Badge variant="neutral">{preview.linkCount} {t.catalog.links}</Badge>
						<Badge variant="neutral">{preview.videoCount} {t.catalog.videos}</Badge>
						<Badge variant="neutral">{preview.noteCount} {t.catalog.notes}</Badge>
						<Badge variant="neutral">{preview.figureCount} {t.catalog.figures}</Badge>
						<Badge variant="neutral">{size(preview.bytes)}</Badge>
					</ul>
					{#if !preview.includesSource}
						<p class="text-xs text-muted-foreground">{t.catalog.sourceExcluded}</p>
					{/if}
					<div class="flex gap-2">
						<Button size="sm" disabled={busy !== ''} onclick={publish}>{t.catalog.confirm}</Button>
						<Button size="sm" variant="ghost" onclick={() => (preview = null)}>
							{t.catalog.cancel}
						</Button>
					</div>
				</div>
			{/if}
		</CardContent>
	</Card>

	<div class="flex flex-wrap items-end gap-2">
		<label class="flex flex-col gap-1 text-sm">
			<span class="text-muted-foreground">{t.catalog.search}</span>
			<Input class="w-64" bind:value={search} placeholder={t.catalog.searchPlaceholder} />
		</label>
		<label class="flex flex-col gap-1 text-sm">
			<span class="text-muted-foreground">{t.catalog.sort}</span>
			<Select class="w-48" bind:value={sort} options={sortOptions} />
		</label>
	</div>

	{#if entries.length === 0}
		<p class="text-sm text-muted-foreground">{search ? t.catalog.noMatch : t.catalog.empty}</p>
	{:else}
		<ul class="flex flex-col gap-2">
			{#each entries as entry (entry.id)}
				<li class="rounded-md border">
					<div class="flex flex-wrap items-center gap-3 px-3 py-2">
						<button
							type="button"
							class="min-w-0 flex-1 text-start"
							onclick={() => open(entry)}
							aria-expanded={openEntry === entry.id}
						>
							<span class="block truncate text-sm font-medium">{entry.title}</span>
							<span class="block truncate text-xs text-muted-foreground">
								{entry.certification} · {t.catalog.by(entry.ownerName)} · {size(entry.bytes)}
							</span>
						</button>

						{#if entry.mine}
							<Badge>{t.catalog.mine}</Badge>
						{/if}
						<Badge variant="neutral">{t.catalog.questions(entry.questionCount)}</Badge>
						{#if entry.needsSourceCount > 0}
							<Badge variant="neutral">{t.catalog.needsSource(entry.needsSourceCount)}</Badge>
						{/if}
						<Badge variant="neutral">
							{entry.rating === null
								? t.catalog.noRating
								: t.catalog.ratingOf(entry.rating, entry.ratingCount)}
						</Badge>

						<Button size="sm" variant="outline" disabled={busy !== ''} onclick={() => take(entry)}>
							<DownloadIcon class="size-4" />
							{busy === entry.id ? t.catalog.importing : t.catalog.import}
						</Button>
						{#if entry.mine}
							<Button
								size="sm"
								variant="ghost"
								aria-label={t.catalog.withdraw}
								disabled={busy !== ''}
								onclick={() => withdraw(entry)}
							>
								<TrashIcon class="size-4" />
							</Button>
						{/if}
					</div>

					{#if openEntry === entry.id}
						<div class="flex flex-col gap-3 border-t px-3 py-3">
							<section class="flex flex-col gap-2">
								<p class="flex items-center gap-1.5 text-sm font-medium">
									<StarIcon class="size-4" />
									{t.catalog.ratings}
								</p>
								{#if ratings.length === 0}
									<p class="text-xs text-muted-foreground">{t.catalog.noRatings}</p>
								{:else}
									<ul class="flex flex-col gap-1 text-xs">
										{#each ratings as rating (rating.raterId)}
											<li class="flex flex-wrap items-baseline gap-2">
												<span class="font-medium">{rating.raterName}</span>
												<span>{'★'.repeat(rating.stars)}</span>
												{#if rating.comment}<span class="text-muted-foreground">
														{rating.comment}
													</span>{/if}
											</li>
										{/each}
									</ul>
								{/if}
								<div class="flex flex-wrap items-end gap-2">
									<label class="flex flex-col gap-1 text-xs">
										<span class="text-muted-foreground">{t.catalog.rating}</span>
										<Select
											class="w-20"
											bind:value={stars}
											options={[1, 2, 3, 4, 5].map((value) => ({
												value: String(value),
												label: String(value)
											}))}
										/>
									</label>
									<Input class="w-72" bind:value={comment} placeholder={t.catalog.comment} />
									<Button size="sm" variant="outline" onclick={() => rate(entry)}>
										{t.catalog.rate}
									</Button>
								</div>
							</section>

							<section class="flex flex-col gap-2">
								<p class="flex items-center gap-1.5 text-sm font-medium">
									<TrophyIcon class="size-4" />
									{t.catalog.board}
								</p>
								{#if seeds.length === 0}
									<p class="text-xs text-muted-foreground">{t.catalog.noBoard}</p>
								{:else}
									<div class="flex flex-wrap gap-1">
										{#each seeds as candidate (candidate)}
											<Button
												size="sm"
												variant={candidate === seed ? 'default' : 'ghost'}
												onclick={() => showBoard(entry.id, candidate)}
											>
												{t.catalog.seed}
												{candidate}
											</Button>
										{/each}
									</div>
									<ol class="flex flex-col gap-1 text-xs">
										{#each board as row, index (row.runnerId + row.finishedAt)}
											<li class="flex flex-wrap items-baseline gap-2">
												<span class="w-4 text-muted-foreground">{index + 1}.</span>
												<span class="font-medium">{row.runnerName}</span>
												<span>{t.catalog.boardRow(row.correct, row.questionCount)}</span>
												<span class="text-muted-foreground">
													{Math.round(row.elapsedMs / 1000)} s
												</span>
											</li>
										{/each}
									</ol>
								{/if}
							</section>
						</div>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}

	<Card>
		<CardHeader>
			<CardTitle>{t.catalog.syncTitle}</CardTitle>
			<CardDescription>{t.catalog.syncBody}</CardDescription>
		</CardHeader>
		<CardContent class="flex flex-col gap-2">
			<div class="flex flex-wrap gap-2">
				<Button variant="outline" disabled={busy !== ''} onclick={() => move('push')}>
					<RefreshIcon class="size-4" />
					{t.catalog.push}
				</Button>
				<Button variant="outline" disabled={busy !== ''} onclick={() => move('pull')}>
					<RefreshIcon class="size-4" />
					{t.catalog.pull}
				</Button>
			</div>
			{#if sync}
				<p class="text-sm">{t.catalog.syncResult(sync.pushed, sync.pulled, sync.skipped)}</p>
			{/if}
			<p class="text-xs text-muted-foreground">{t.catalog.oneMachine}</p>
		</CardContent>
	</Card>
</div>
