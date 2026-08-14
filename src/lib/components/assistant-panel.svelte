<script lang="ts">
	import SparklesIcon from '@lucide/svelte/icons/sparkles';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { call } from '$lib/bridge/client';
	import type { AssistantStatus, AssistantTask, Question } from '$lib/bridge/contract';
	import { settings } from '$lib/stores/settings.svelte';
	import { notes } from '$lib/stores/notes.svelte';
	import { library } from '$lib/stores/library.svelte';
	import { i18n } from '$lib/i18n/index.svelte';

	const t = $derived(i18n.t);

	interface Props {
		question: Question;
	}

	let { question }: Props = $props();

	let status = $state<AssistantStatus | null>(null);
	let answer = $state<string | null>(null);
	let error = $state<string | null>(null);
	let busy = $state(false);
	let kept = $state(false);

	const TASKS = $derived<{ id: AssistantTask; label: string }[]>([
		{ id: 'explain', label: t.assistant.explain },
		{ id: 'variants', label: t.assistant.variants },
		{ id: 'note', label: t.assistant.note }
	]);

	$effect(() => {
		const source = settings.assistantSource;
		call('assistant_status', { source })
			.then((result) => (status = result))
			.catch(() => (status = null));
	});

	// A new question invalidates the old answer; leaving it on screen would attach an explanation
	// to the wrong question.
	$effect(() => {
		void question.id;
		answer = null;
		error = null;
		kept = false;
	});

	async function keep() {
		const binderId = library.selectedId;
		if (binderId === null || !answer) return;
		await notes.save(binderId, question.id, answer);
		kept = true;
	}

	const ready = $derived(
		settings.assistantSource === 'cli' ? (status?.cliAvailable ?? false) : (status?.hasKey ?? false)
	);

	async function ask(task: AssistantTask) {
		busy = true;
		error = null;
		answer = null;
		try {
			answer = await call('assistant_ask', {
				source: settings.assistantSource,
				task,
				questionId: question.id
			});
		} catch (caught) {
			error = caught instanceof Error ? caught.message : String(caught);
		} finally {
			busy = false;
		}
	}
</script>

<div class="flex flex-col gap-3 rounded-md border p-4">
	<div class="flex items-center gap-2 text-sm font-medium">
		<SparklesIcon class="size-4" />
		{t.assistant.title}
		<Badge variant="neutral">
			{settings.assistantSource === 'cli' ? t.assistant.sourceCli : t.assistant.sourceAnthropic}
		</Badge>
	</div>

	{#if !ready}
		<p class="text-xs text-muted-foreground">
			{#if settings.assistantSource === 'cli'}
				{t.assistant.noCli}
			{:else}
				{t.assistant.noKey}
			{/if}
		</p>
	{/if}

	<div class="flex flex-wrap gap-2">
		{#each TASKS as task (task.id)}
			<Button size="sm" variant="outline" disabled={busy || !ready} onclick={() => ask(task.id)}>
				{task.label}
			</Button>
		{/each}
	</div>

	{#if busy}
		<p class="text-sm text-muted-foreground">{t.assistant.thinking}</p>
	{/if}
	{#if error}
		<p class="text-sm whitespace-pre-wrap text-destructive">{error}</p>
	{/if}
	{#if answer}
		<p class="text-sm whitespace-pre-wrap">{answer}</p>
		<Button size="sm" variant="outline" class="self-start" onclick={keep} disabled={kept}>
			{kept ? t.notes.saved : t.notes.saveAnswer}
		</Button>
	{/if}
</div>
