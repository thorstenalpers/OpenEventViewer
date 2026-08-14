<script lang="ts">
	import NotebookIcon from '@lucide/svelte/icons/notebook-pen';
	import { Button } from '$lib/components/ui/button';
	import { Textarea } from '$lib/components/ui/textarea';
	import { notes } from '$lib/stores/notes.svelte';
	import { library } from '$lib/stores/library.svelte';
	import { i18n } from '$lib/i18n/index.svelte';

	interface Props {
		questionId: number;
	}

	let { questionId }: Props = $props();

	const t = $derived(i18n.t);
	const existing = $derived(notes.forQuestion(questionId));

	let draft = $state('');
	let saving = $state(false);

	async function save() {
		const binderId = library.selectedId;
		if (binderId === null || !draft.trim()) return;
		saving = true;
		await notes.save(binderId, questionId, draft);
		draft = '';
		saving = false;
	}
</script>

<div class="flex flex-col gap-2">
	<p class="flex items-center gap-2 text-xs font-medium text-muted-foreground">
		<NotebookIcon class="size-3.5" />
		{t.notes.title}
	</p>

	{#each existing as note (note.id)}
		<p class="rounded-md border bg-muted/30 px-3 py-2 text-sm whitespace-pre-wrap">
			{note.bodyMd}
		</p>
	{:else}
		<p class="text-xs text-muted-foreground">{t.notes.none}</p>
	{/each}

	<Textarea bind:value={draft} placeholder={t.notes.placeholder} rows={3} />
	<Button size="sm" class="self-start" onclick={save} disabled={saving || !draft.trim()}>
		{t.notes.save}
	</Button>

	{#if notes.error}
		<p class="text-sm text-destructive">{notes.error}</p>
	{/if}
</div>
