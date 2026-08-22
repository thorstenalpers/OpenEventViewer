<script lang="ts">
	import SendIcon from '@lucide/svelte/icons/send';
	import XIcon from '@lucide/svelte/icons/x';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import { Button } from '$lib/components/ui/button';
	import { Textarea } from '$lib/components/ui/textarea';
	import AssistantStatusBadge from '$lib/components/assistant-status-badge.svelte';
	import { i18n } from '$lib/i18n/index.svelte';
	import { cn } from '$lib/utils';
	import { assistant } from '$lib/stores/assistant.svelte';

	const t = $derived(i18n.t);

	const next = $derived(assistant.composeNext());
	// Open by default once something is attached: that is the case where what would be sent is more
	// than what the user typed, and the only one where not looking is a real risk.
	let showPreview = $state(false);
	const expanded = $derived(showPreview || assistant.attachments.length > 0);
</script>

<div class="flex h-full flex-col gap-3 p-4 sm:p-6">
	<div class="flex flex-wrap items-center justify-between gap-2">
		<AssistantStatusBadge />
		<Button
			size="sm"
			variant="ghost"
			onclick={() => assistant.reset()}
			disabled={assistant.busy || assistant.messages.length === 0}
		>
			{t.assistant.newConversation}
		</Button>
	</div>

	<div class="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto">
		{#if assistant.messages.length === 0}
			<p class="text-sm text-muted-foreground">{t.assistant.empty}</p>
		{/if}
		{#each assistant.messages as message, index (index)}
			<div
				class={cn(
					'max-w-3xl rounded-md border px-3 py-2 text-sm whitespace-pre-wrap',
					message.role === 'user' ? 'self-end bg-muted/50' : 'self-start'
				)}
			>
				{message.content}
			</div>
		{/each}
		{#if assistant.busy}
			<p class="text-sm text-muted-foreground">{t.assistant.thinking}</p>
		{/if}
		{#if assistant.error}
			<p class="text-sm whitespace-pre-wrap text-destructive">{assistant.error}</p>
		{/if}
	</div>

	{#if assistant.attachments.length}
		<div class="flex flex-wrap gap-1.5">
			{#each assistant.attachments as attachment (attachment.id)}
				<span
					class="flex items-center gap-1 rounded-md border border-primary/40 px-2 py-0.5 text-xs text-primary"
				>
					{attachment.title}
					<span class="text-muted-foreground">
						{t.assistant.attachedCount(attachment.events.length)}
					</span>
					<button
						type="button"
						class="cursor-pointer rounded hover:text-destructive"
						aria-label={t.assistant.removeAttachment(attachment.title)}
						onclick={() => assistant.remove(attachment.id)}
					>
						<XIcon class="size-3" />
					</button>
				</span>
			{/each}
		</div>
	{/if}

	<div class="flex flex-col gap-2 rounded-md border p-3">
		<div class="flex items-center justify-between gap-2">
			<button
				type="button"
				class="flex cursor-pointer items-center gap-1.5 text-xs font-medium text-muted-foreground hover:text-foreground"
				aria-expanded={expanded}
				onclick={() => (showPreview = !expanded)}
			>
				<EyeIcon class="size-3.5" />
				{t.assistant.preview}
			</button>
			<span class="text-xs text-muted-foreground">{t.assistant.characters(next.length)}</span>
		</div>
		<p class="text-xs text-muted-foreground">{t.assistant.previewBody}</p>
		{#if expanded}
			<div class="flex flex-col gap-2">
				<div>
					<p class="pb-1 text-[11px] font-medium text-muted-foreground">
						{t.assistant.systemPrompt}
					</p>
					<pre
						class="max-h-32 overflow-auto rounded bg-muted/50 p-2 text-[11px] whitespace-pre-wrap">{assistant.systemPrompt}</pre>
				</div>
				<div>
					<p class="pb-1 text-[11px] font-medium text-muted-foreground">
						{t.assistant.nextMessage}
					</p>
					<pre
						data-testid="preview"
						class="max-h-64 overflow-auto rounded bg-muted/50 p-2 text-[11px] whitespace-pre-wrap">{next ||
							t.assistant.nothingYet}</pre>
				</div>
			</div>
		{/if}
	</div>

	<div class="flex items-end gap-2">
		<Textarea
			bind:value={assistant.draft}
			placeholder={t.assistant.placeholder}
			aria-label={t.assistant.placeholder}
			rows={3}
			class="flex-1"
		/>
		<Button
			onclick={() => assistant.send()}
			disabled={assistant.busy || !assistant.ready || next.trim().length === 0}
		>
			<SendIcon class="size-4" />
			{t.assistant.send}
		</Button>
	</div>
</div>
