<script lang="ts">
	import PlayIcon from '@lucide/svelte/icons/play';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import TrashIcon from '@lucide/svelte/icons/trash-2';
	import ExternalIcon from '@lucide/svelte/icons/external-link';
	import FilmIcon from '@lucide/svelte/icons/film';
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
	import { call, isMockHost } from '$lib/bridge/client';
	import type { Link, LinkKind } from '$lib/bridge/contract';
	import { library } from '$lib/stores/library.svelte';
	import { i18n } from '$lib/i18n/index.svelte';
	import NoBinder from '$lib/components/no-binder.svelte';

	const t = $derived(i18n.t);

	const binder = $derived(library.selected);

	let links = $state<Link[]>([]);
	let error = $state<string | null>(null);

	/** Which shelf is open. Empty means everything, which is what someone arriving wants to see. */
	let shelf = $state<'' | LinkKind>('');
	let playing = $state<string | null>(null);

	let form = $state<{
		url: string;
		title: string;
		description: string;
		kind: LinkKind;
		minutes: string;
	}>({ url: '', title: '', description: '', kind: 'course', minutes: '' });

	const KINDS = ['course', 'video', 'docs', 'other'] as const;

	/** The dictionary is typed as open-ended so a stored kind nobody knows still renders. */
	function kindLabel(kind: string): string {
		return t.study.kinds[kind] ?? kind;
	}

	const kindOptions = $derived(KINDS.map((kind) => ({ value: kind, label: kindLabel(kind) })));

	$effect(() => {
		const id = binder?.id;
		if (id === undefined) {
			links = [];
			return;
		}
		call('list_links', { binderId: id })
			.then((result) => (links = result))
			.catch(() => (links = []));
	});

	/** The shelves, each with what it holds — a count is the only honest label for an empty one. */
	const shelves = $derived([
		{ id: '' as const, label: t.study.all, count: links.length },
		...KINDS.map((kind) => ({
			id: kind,
			label: kindLabel(kind),
			count: links.filter((link) => link.kind === kind).length
		}))
	]);

	const shown = $derived(shelf === '' ? links : links.filter((link) => link.kind === shelf));

	const totalMinutes = $derived(shown.reduce((sum, link) => sum + (link.minutes ?? 0), 0));

	/** A local file is stored as its path; everything else is an address the browser can open. */
	function isLocal(link: Link): boolean {
		return !/^https?:/i.test(link.url);
	}

	async function add(event: SubmitEvent) {
		event.preventDefault();
		const id = binder?.id;
		if (id === undefined || !form.url.trim()) return;
		error = null;
		try {
			links = await call('save_link', {
				binderId: id,
				link: {
					questionId: null,
					url: form.url.trim(),
					title: form.title.trim() || form.url.trim(),
					description: form.description.trim(),
					kind: form.kind,
					minutes: form.minutes ? Number(form.minutes) : null
				}
			});
			form = { url: '', title: '', description: '', kind: form.kind, minutes: '' };
		} catch (caught) {
			error = caught instanceof Error ? caught.message : String(caught);
		}
	}

	/** Picks a video off this machine. Its path is the link's address; nothing is copied. */
	async function addVideo() {
		const id = binder?.id;
		if (id === undefined) return;
		const picked = isMockHost()
			? 'C:\\videos\\az-900-intro.mp4'
			: await (
					await import('@tauri-apps/plugin-dialog')
				).open({
					multiple: false,
					filters: [{ name: 'Video', extensions: ['mp4', 'webm', 'mkv', 'mov'] }]
				});
		if (typeof picked !== 'string') return;

		const name = picked.split(/[\\/]/).pop() ?? picked;
		links = await call('save_link', {
			binderId: id,
			link: {
				questionId: null,
				url: picked,
				title: name,
				description: '',
				kind: 'video',
				minutes: null
			}
		}).catch(() => links);
	}

	async function remove(link: Link) {
		const id = binder?.id;
		if (id === undefined) return;
		if (playing === link.url) playing = null;
		links = await call('delete_link', { binderId: id, linkId: link.id }).catch(() => links);
	}

	/**
	 * A local file cannot be handed to a `<video>` element by path — the webview refuses `file:`.
	 * Tauri's asset protocol is the door, and `convertFileSrc` is the key it takes.
	 */
	function playable(path: string): string {
		if (isMockHost()) return '';
		return convert?.(path) ?? '';
	}

	let convert = $state<((path: string) => string) | null>(null);

	$effect(() => {
		if (isMockHost()) return;
		void import('@tauri-apps/api/core').then((core) => {
			convert = (path: string) => core.convertFileSrc(path);
		});
	});

	function duration(minutes: number | null): string {
		if (!minutes) return '';
		return minutes >= 60
			? t.study.hours(Math.floor(minutes / 60), minutes % 60)
			: t.study.minutes(minutes);
	}
</script>

<div class="flex flex-col gap-3 p-4 sm:p-6">
	<header>
		<h1 class="text-xl font-semibold">{t.study.title}</h1>
		<p class="text-sm text-muted-foreground">
			{#if binder}{t.study.subtitle(binder.title)}{/if}
		</p>
	</header>

	{#if !binder}
		<NoBinder />
	{:else}
		{#if error}
			<p class="text-sm text-destructive">{error}</p>
		{/if}

		<!-- The shelves are the submenu: one row of counts, and the list below answers to it. -->
		<div class="flex flex-wrap items-center gap-1.5">
			{#each shelves as entry (entry.id)}
				<button
					type="button"
					onclick={() => (shelf = entry.id)}
					class="flex cursor-pointer items-center gap-1.5 rounded-md border px-2.5 py-1 text-xs transition-colors
					{shelf === entry.id ? 'border-primary bg-primary/10 font-medium' : 'hover:bg-muted/60'}"
				>
					{entry.label}
					<span class="text-muted-foreground tabular-nums">{entry.count}</span>
				</button>
			{/each}
			{#if totalMinutes > 0}
				<span class="ms-auto text-xs text-muted-foreground">
					{t.study.totalTime(duration(totalMinutes))}
				</span>
			{/if}
		</div>

		{#if shown.length === 0}
			<p class="text-sm text-muted-foreground">{t.study.empty}</p>
		{:else}
			<ul class="flex flex-col gap-2">
				{#each shown as link (link.id)}
					<li class="flex flex-col gap-2 rounded-md border px-3 py-2">
						<div class="flex flex-wrap items-center gap-3">
							<Badge variant="neutral">{kindLabel(link.kind)}</Badge>
							<span class="min-w-0 flex-1 truncate text-sm font-medium">{link.title}</span>
							{#if link.minutes}
								<span class="text-xs text-muted-foreground tabular-nums">
									{duration(link.minutes)}
								</span>
							{/if}
							{#if isLocal(link)}
								<Button
									size="sm"
									variant="outline"
									onclick={() => (playing = playing === link.url ? null : link.url)}
								>
									<PlayIcon class="size-4" />
									{playing === link.url ? t.study.close : t.study.play}
								</Button>
							{:else}
								<Button
									size="sm"
									variant="outline"
									href={link.url}
									target="_blank"
									rel="external noreferrer"
								>
									<ExternalIcon class="size-4" />
									{t.study.open}
								</Button>
							{/if}
							<Button
								size="sm"
								variant="ghost"
								aria-label={t.study.removeAria(link.title)}
								onclick={() => remove(link)}
							>
								<TrashIcon class="size-4" />
							</Button>
						</div>

						{#if link.description}
							<p class="text-xs text-muted-foreground">{link.description}</p>
						{/if}
						{#if !isLocal(link)}
							<p class="truncate text-xs text-muted-foreground">{link.url}</p>
						{/if}

						{#if playing === link.url}
							{@const source = playable(link.url)}
							{#if source}
								<!-- svelte-ignore a11y_media_has_caption -->
								<video src={source} controls class="w-full rounded-md border"></video>
							{:else}
								<p class="text-xs text-muted-foreground">{t.study.noPlayback}</p>
							{/if}
						{/if}
					</li>
				{/each}
			</ul>
		{/if}

		<Card>
			<CardHeader>
				<CardTitle>{t.study.addTitle}</CardTitle>
				<CardDescription>{t.study.addBody}</CardDescription>
			</CardHeader>
			<CardContent class="flex flex-col gap-2">
				<form
					class="grid grid-cols-1 items-end gap-2 sm:grid-cols-[2fr_1fr_8rem_6rem_auto]"
					onsubmit={add}
				>
					<label class="flex flex-col gap-1 text-xs text-muted-foreground">
						{t.study.url}
						<Input bind:value={form.url} placeholder="https://www.youtube.com/watch?v=…" required />
					</label>
					<label class="flex flex-col gap-1 text-xs text-muted-foreground">
						{t.study.linkTitle}
						<Input bind:value={form.title} placeholder="AZ-900 Full Course" />
					</label>
					<span class="flex flex-col gap-1 text-xs text-muted-foreground select-none">
						{t.study.kind}
						<Select bind:value={form.kind} options={kindOptions} aria-label={t.study.kind} />
					</span>
					<label class="flex flex-col gap-1 text-xs text-muted-foreground">
						{t.study.minutesLabel}
						<Input type="number" min="0" bind:value={form.minutes} placeholder="150" />
					</label>
					<Button type="submit">
						<PlusIcon class="size-4" />
						{t.study.add}
					</Button>
				</form>

				<label class="flex flex-col gap-1 text-xs text-muted-foreground">
					{t.study.description}
					<Input bind:value={form.description} placeholder={t.study.descriptionPlaceholder} />
				</label>

				<Button variant="outline" class="self-start" onclick={addVideo}>
					<FilmIcon class="size-4" />
					{t.study.addVideo}
				</Button>
			</CardContent>
		</Card>
	{/if}
</div>
