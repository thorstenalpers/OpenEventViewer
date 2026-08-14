<script lang="ts">
	import type { ColumnDef } from '@tanstack/table-core';
	import ArrowUpIcon from '@lucide/svelte/icons/arrow-up';
	import ArrowDownIcon from '@lucide/svelte/icons/arrow-down';
	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
	import TrashIcon from '@lucide/svelte/icons/trash-2';
	import DownloadIcon from '@lucide/svelte/icons/download';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import FileInputIcon from '@lucide/svelte/icons/file-input';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import {
		Table,
		TableBody,
		TableCell,
		TableHead,
		TableHeader,
		TableRow
	} from '$lib/components/ui/table';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { createDataTable } from '$lib/table.svelte';
	import { Select } from '$lib/components/ui/select';
	import { library } from '$lib/stores/library.svelte';
	import { viewState } from '$lib/stores/view-state.svelte';
	import { i18n } from '$lib/i18n/index.svelte';
	import type { Binder, Template } from '$lib/bridge/contract';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { open, save } from '@tauri-apps/plugin-dialog';
	import { call, isMockHost } from '$lib/bridge/client';

	const t = $derived(i18n.t);

	const columns: ColumnDef<Binder, never>[] = [
		{ accessorKey: 'title', header: 'project' },
		{ accessorKey: 'certification', header: 'certification' },
		{ accessorKey: 'questionCount', header: 'questions' },
		{ accessorKey: 'needsReviewCount', header: 'review' },
		{ accessorKey: 'attemptCount', header: 'attempts' },
		{ accessorKey: 'accuracy', header: 'accuracy' },
		{ accessorKey: 'importedAt', header: 'created' },
		{ accessorKey: 'lastStudiedAt', header: 'lastStudied' }
	];

	// Newest first: the project you just made is the one you want. Keyed, so a trip to Train and
	// back returns to the same sort and filter rather than to the default.
	const data = createDataTable<Binder>(
		() => library.binders,
		columns,
		[{ id: 'importedAt', desc: true }],
		'projects'
	);

	let error = $state<string | null>(null);
	let notice = $state<string | null>(null);
	let importing = $state(false);

	// In a store: importing, glancing at Train and coming back should still show the report.
	const report = $derived(viewState.importReport);

	let creating = $state(false);
	let newTitle = $state('');
	let newCode = $state('');

	let templates = $state<Template[]>([]);
	// Empty means "not from the catalogue" — the code and the doc URL are then typed by hand.
	let chosenTemplate = $state('');
	let newDocUrl = $state('');

	$effect(() => {
		call('list_templates', {})
			.then((result) => (templates = result))
			.catch(() => (templates = []));
	});

	const templateOptions = $derived([
		{ value: '', label: t.projects.ownExam },
		...templates.map((template) => ({ value: String(template.id), label: template.name }))
	]);

	/** Picking a template fills the code and the documentation address; both stay editable. */
	function useTemplate(value: string) {
		chosenTemplate = value;
		const template = templates.find((entry) => String(entry.id) === value);
		if (!template) return;
		newCode = template.name;
		newDocUrl = template.docUrl;
	}

	function formatDate(value: string | null): string {
		return value ? new Date(value).toLocaleDateString(i18n.locale) : '—';
	}

	function formatAccuracy(value: number | null): string {
		return value === null ? '—' : `${Math.round(value * 100)}%`;
	}

	const isEmpty = (project: Binder) => project.questionCount === 0;

	async function create(event: SubmitEvent) {
		event.preventDefault();
		if (!newCode.trim()) return;
		error = null;
		try {
			const created = await call('create_project', {
				title: newTitle.trim() || newCode.trim(),
				certification: newCode.trim(),
				docUrl: newDocUrl.trim()
			});
			// A code typed by hand becomes a template, so the second project of that kind is a pick.
			if (newDocUrl.trim() && !chosenTemplate) {
				templates = await call('save_template', {
					name: newCode.trim(),
					docUrl: newDocUrl.trim()
				});
			}
			await library.refresh();
			library.selectedId = created.id;
			newTitle = '';
			newCode = '';
			newDocUrl = '';
			chosenTemplate = '';
			creating = false;
			notice = t.projects.created(created.certification);
		} catch (caught) {
			error = caught instanceof Error ? caught.message : String(caught);
		}
	}

	async function openExam(project: Binder) {
		library.selectedId = project.id;
		await goto(resolve('/exam'));
	}

	async function train(project: Binder) {
		library.selectedId = project.id;
		await goto(resolve('/train'));
	}

	let mockPicks = 0;

	async function pickMaterial(): Promise<string | null> {
		// Alternating in the mock is what makes both report shapes reachable in the browser: a PDF
		// reports pages and figures, a question bank reports neither.
		if (isMockHost()) {
			mockPicks += 1;
			return mockPicks % 2 === 1
				? 'C:\\fixtures\\certleader-ai900.pdf'
				: 'C:\\fixtures\\ai-900-practice-questions.md';
		}
		const picked = await open({
			multiple: false,
			filters: [
				{ name: 'Exam material', extensions: ['pdf', 'vce', 'md', 'markdown', 'json'] },
				{ name: 'Exam dump (PDF)', extensions: ['pdf'] },
				{ name: 'Question bank', extensions: ['md', 'markdown', 'json'] }
			]
		});
		return typeof picked === 'string' ? picked : null;
	}

	/**
	 * Reads exam material into a project.
	 *
	 * Importing into a project that already holds questions would merge two exams into one score,
	 * so an existing project is only offered as the target while it is still empty; everything else
	 * lands in a project the extractor names after the file.
	 */
	async function importMaterial(project?: Binder) {
		error = null;
		notice = null;
		viewState.importReport = null;
		const path = await pickMaterial();
		if (!path) return;

		if (project) library.selectedId = project.id;
		const target = project ?? library.selected;
		const projectId = target && target.questionCount === 0 ? target.id : undefined;

		importing = true;
		try {
			viewState.importReport = await call('import_file', { path, projectId });
			await library.refresh();
			library.selectedId = viewState.importReport.binder.id;
		} catch (caught) {
			error = caught instanceof Error ? caught.message : String(caught);
		} finally {
			importing = false;
		}
	}

	async function exportDeck(project: Binder) {
		error = null;
		notice = null;
		const suggested = `${project.title}.examdeck`;
		const path = isMockHost()
			? `C:\\mock\\${suggested}`
			: await save({
					defaultPath: suggested,
					filters: [{ name: 'Project', extensions: ['examdeck'] }]
				});
		if (!path) return;
		try {
			const manifest = await call('export_deck', { binderId: project.id, path });
			notice = t.projects.exported(manifest.questionCount, path);
		} catch (caught) {
			error = caught instanceof Error ? caught.message : String(caught);
		}
	}

	async function importDeck() {
		error = null;
		notice = null;
		const picked = isMockHost()
			? 'C:\\mock\\AI-900.examdeck'
			: await open({
					multiple: false,
					filters: [{ name: 'Project', extensions: ['examdeck'] }]
				});
		if (typeof picked !== 'string') return;
		try {
			const preview = await call('peek_deck', { path: picked });
			const imported = await call('import_deck', { path: picked });
			await library.refresh();
			library.selectedId = imported.id;
			notice = t.projects.imported(preview.title, preview.questionCount);
		} catch (caught) {
			error = caught instanceof Error ? caught.message : String(caught);
		}
	}
</script>

<div class="flex flex-col gap-3 p-4 sm:p-6">
	<header class="flex flex-wrap items-end justify-between gap-4">
		<div>
			<h1 class="text-xl font-semibold">{t.projects.title}</h1>
			<p class="text-sm text-muted-foreground">{t.projects.subtitle(library.binders.length)}</p>
		</div>
		<div class="flex flex-wrap gap-2">
			<Button variant="outline" onclick={importDeck}>{t.projects.importDeck}</Button>
			<Button variant="outline" onclick={() => importMaterial()} disabled={importing}>
				<FileInputIcon class="size-4" />
				{importing ? t.import.extracting : t.import.choose}
			</Button>
			<Button onclick={() => (creating = !creating)}>
				<PlusIcon class="size-4" />
				{t.projects.create}
			</Button>
		</div>
	</header>

	<p class="max-w-3xl text-xs text-muted-foreground">{t.import.hint}</p>

	{#if creating}
		<form class="flex flex-col gap-2 rounded-md border p-4" onsubmit={create}>
			<div class="grid grid-cols-1 items-end gap-2 sm:grid-cols-[12rem_8rem_1fr_auto]">
				<label class="flex flex-col gap-1 text-xs text-muted-foreground">
					{t.projects.template}
					<Select
						value={chosenTemplate}
						options={templateOptions}
						onchange={(event: Event) =>
							useTemplate((event.currentTarget as HTMLSelectElement).value)}
						aria-label={t.projects.template}
					/>
				</label>
				<label class="flex flex-col gap-1 text-xs text-muted-foreground">
					{t.projects.code}
					<Input bind:value={newCode} placeholder="AI-102" required />
				</label>
				<label class="flex flex-col gap-1 text-xs text-muted-foreground">
					{t.projects.name}
					<Input bind:value={newTitle} placeholder={t.projects.namePlaceholder} />
				</label>
				<Button type="submit">{t.projects.save}</Button>
			</div>
			<label class="flex flex-col gap-1 text-xs text-muted-foreground">
				{t.projects.docUrl}
				<Input
					bind:value={newDocUrl}
					placeholder="https://learn.microsoft.com/en-us/credentials/certifications/…"
				/>
			</label>
			<!-- Name and address together are what makes an exam that exam, so both are shown even
			     when the template filled them in: an exam retired and replaced keeps the code. -->
			<p class="text-xs text-muted-foreground">{t.projects.templateHint}</p>
		</form>
	{/if}

	{#if error}
		<p class="text-sm text-destructive">{error}</p>
	{:else if notice}
		<p class="text-sm text-muted-foreground">{notice}</p>
	{/if}

	{#if report}
		<Card>
			<CardHeader>
				<CardTitle>{report.binder.title}</CardTitle>
				<CardDescription>
					{t.import.profile}
					<Badge variant="accent">{report.profile}</Badge>
					{#if report.pages}
						· {t.import.meta(report.pages, report.furnitureDropped)}
					{/if}
				</CardDescription>
			</CardHeader>
			<CardContent class="flex flex-col gap-3">
				<!-- A question bank has no pages, so the figure columns would be two zeroes that mean
				     "not applicable" rather than "none found". -->
				<dl class={`grid grid-cols-2 gap-3 ${report.pages ? 'sm:grid-cols-4' : ''}`}>
					<div>
						<dt class="text-xs text-muted-foreground">{t.import.questions}</dt>
						<dd class="text-2xl font-semibold">{report.binder.questionCount}</dd>
					</div>
					<div>
						<dt class="text-xs text-muted-foreground">{t.import.needReview}</dt>
						<dd class="text-2xl font-semibold">{report.binder.needsReviewCount}</dd>
					</div>
					{#if report.pages}
						<div>
							<dt class="text-xs text-muted-foreground">{t.import.figuresRecovered}</dt>
							<dd class="text-2xl font-semibold">{report.figuresRecovered}</dd>
						</div>
						<div>
							<dt class="text-xs text-muted-foreground">{t.import.missingFigure}</dt>
							<dd class="text-2xl font-semibold">{report.binder.needsSourceCount}</dd>
						</div>
					{/if}
				</dl>

				{#if report.stubMarkers.length}
					<p class="rounded-md bg-warning/10 px-3 py-2 text-sm">
						{t.import.excerpt(report.stubMarkers)}
					</p>
				{/if}

				{#if report.missingNumbers.length}
					<p class="text-sm text-muted-foreground">{t.import.skips(report.missingNumbers)}</p>
				{/if}

				<div class="flex flex-wrap gap-2">
					<Button href={resolve('/train')}>{t.import.startTraining}</Button>
					{#if report.binder.needsReviewCount > 0}
						<Button href={resolve('/review')} variant="outline">
							{t.import.review(report.binder.needsReviewCount)}
						</Button>
					{/if}
				</div>
			</CardContent>
		</Card>
	{/if}

	<div class="flex flex-wrap items-center gap-3">
		<Input placeholder={t.projects.filter} bind:value={data.globalFilter} class="max-w-sm" />
		<p class="text-xs text-muted-foreground">{t.projects.multiSortHint}</p>
	</div>

	{#if library.error}
		<p class="text-sm text-destructive">{library.error}</p>
	{/if}

	<div class="rounded-md border">
		<Table>
			<TableHeader>
				{#each data.table.getHeaderGroups() as headerGroup (headerGroup.id)}
					<TableRow>
						{#each headerGroup.headers as header (header.id)}
							{@const sorted = header.column.getIsSorted()}
							{@const index = header.column.getSortIndex()}
							<TableHead>
								<button
									type="button"
									class="flex cursor-pointer items-center gap-1 hover:text-foreground"
									onclick={header.column.getToggleSortingHandler()}
								>
									{t.projects.columns[
										header.column.columnDef.header as keyof typeof t.projects.columns
									]}
									{#if sorted === 'asc'}
										<ArrowUpIcon class="size-3" />
									{:else if sorted === 'desc'}
										<ArrowDownIcon class="size-3" />
									{:else}
										<ChevronsUpDownIcon class="size-3 opacity-40" />
									{/if}
									<!-- The rank only means something once a second column joins the sort. -->
									{#if sorted && index > -1 && data.table.getState().sorting.length > 1}
										<span class="text-[10px] text-muted-foreground tabular-nums">{index + 1}</span>
									{/if}
								</button>
							</TableHead>
						{/each}
						<TableHead class="w-32 text-end">{t.projects.columns.actions}</TableHead>
					</TableRow>
				{/each}
			</TableHeader>
			<TableBody>
				{#each data.table.getRowModel().rows as row (row.id)}
					{@const project = row.original}
					<TableRow
						data-state={project.id === library.selectedId ? 'selected' : undefined}
						class="cursor-pointer"
						onclick={() => (library.selectedId = project.id)}
					>
						<TableCell class="font-medium">
							<button
								type="button"
								class="cursor-pointer text-start hover:underline"
								onclick={() => openExam(project)}
							>
								{project.title}
							</button>
						</TableCell>
						<TableCell><Badge variant="accent">{project.certification}</Badge></TableCell>
						<TableCell>
							{#if isEmpty(project)}
								<span class="text-muted-foreground">{t.projects.noFile}</span>
							{:else}
								{project.questionCount}
							{/if}
						</TableCell>
						<TableCell>
							{#if project.needsReviewCount > 0}
								<a href={resolve('/review')} class="underline underline-offset-2">
									{project.needsReviewCount}
								</a>
							{:else}
								—
							{/if}
						</TableCell>
						<TableCell>{project.attemptCount}</TableCell>
						<TableCell>{formatAccuracy(project.accuracy)}</TableCell>
						<TableCell>{formatDate(project.importedAt)}</TableCell>
						<TableCell>{formatDate(project.lastStudiedAt)}</TableCell>
						<TableCell class="text-end">
							<div class="flex justify-end gap-1">
								{#if isEmpty(project)}
									<Button
										size="sm"
										variant="outline"
										disabled={importing}
										onclick={() => importMaterial(project)}
									>
										{t.projects.addFile}
									</Button>
								{:else}
									<Button size="sm" variant="outline" onclick={() => train(project)}>
										{t.projects.train}
									</Button>
									<Button
										size="sm"
										variant="ghost"
										aria-label={t.projects.exportAria(project.title)}
										onclick={() => exportDeck(project)}
									>
										<DownloadIcon class="size-4" />
									</Button>
								{/if}
								<Button
									size="sm"
									variant="ghost"
									aria-label={t.projects.deleteAria(project.title)}
									onclick={() => library.remove(project.id)}
								>
									<TrashIcon class="size-4" />
								</Button>
							</div>
						</TableCell>
					</TableRow>
				{:else}
					<TableRow>
						<TableCell colspan={columns.length + 1} class="h-24 text-center text-muted-foreground">
							{library.loading ? t.common.loading : t.projects.empty}
						</TableCell>
					</TableRow>
				{/each}
			</TableBody>
		</Table>
	</div>
</div>
