import { call } from '$lib/bridge/client';
import type { AssistantStatus, ChatMessage, EventRecord } from '$lib/bridge/contract';
import { settings } from '$lib/stores/settings.svelte';

export interface Attachment {
	id: string;
	kind: 'event' | 'bundle';
	title: string;
	/** Exactly the text the host rendered, so the preview and the send cannot disagree. */
	text: string;
	events: EventRecord[];
}

/** The heading the attached text is filed under, in the prompt and in the preview alike. */
const HEADING = '--- Attached events ---';

/**
 * The conversation, its attachments, and the one string both the preview and the send use.
 *
 * A singleton because the Events page hands it an attachment and then navigates: the store has to
 * outlive the page that filled it.
 */
class AssistantStore {
	messages = $state<ChatMessage[]>([]);
	attachments = $state<Attachment[]>([]);
	draft = $state('');
	busy = $state(false);
	error = $state<string | null>(null);
	status = $state<AssistantStatus | null>(null);

	get ready(): boolean {
		return settings.assistantSource === 'cli'
			? (this.status?.cliAvailable ?? false)
			: (this.status?.hasKey ?? false);
	}

	get systemPrompt(): string {
		return this.status?.systemPrompt ?? '';
	}

	/**
	 * The next message, in full.
	 *
	 * Pure and the only place this text is built. The preview renders what this returns and `send`
	 * posts what this returns — nothing may be added between the two.
	 */
	composeNext(): string {
		const draft = this.draft.trim();
		if (!this.attachments.length) return draft;
		const attached = this.attachments.map((attachment) => attachment.text.trim()).join('\n\n');
		return draft ? `${draft}\n\n${HEADING}\n${attached}` : `${HEADING}\n${attached}`;
	}

	async refreshStatus(): Promise<void> {
		try {
			this.status = await call('assistant_status', { source: settings.assistantSource });
		} catch {
			this.status = null;
		}
	}

	async attachEvent(event: EventRecord): Promise<void> {
		const id = `event:${event.channel}:${event.recordId}`;
		if (this.attachments.some((attachment) => attachment.id === id)) return;
		const text = await call('events_render', { events: [event] });
		this.attachments = [
			...this.attachments,
			{
				id,
				kind: 'event',
				title: `${event.provider} ${event.eventId}`,
				text,
				events: [event]
			}
		];
	}

	attach(attachment: Attachment): void {
		if (this.attachments.some((held) => held.id === attachment.id)) return;
		this.attachments = [...this.attachments, attachment];
	}

	remove(id: string): void {
		this.attachments = this.attachments.filter((attachment) => attachment.id !== id);
	}

	async send(): Promise<void> {
		const content = this.composeNext();
		if (!content.trim() || this.busy) return;

		this.busy = true;
		this.error = null;
		// Cleared before the reply arrives: what was attached is now part of the transcript, and
		// leaving the chips up would attach the same events again on the next question.
		this.messages = [...this.messages, { role: 'user', content }];
		this.draft = '';
		this.attachments = [];

		try {
			const reply = await call('assistant_chat', {
				source: settings.assistantSource,
				messages: $state.snapshot(this.messages)
			});
			this.messages = [...this.messages, { role: 'assistant', content: reply }];
		} catch (error) {
			this.error = error instanceof Error ? error.message : String(error);
		} finally {
			this.busy = false;
		}
	}

	reset(): void {
		this.messages = [];
		this.attachments = [];
		this.draft = '';
		this.error = null;
	}
}

export const assistant = new AssistantStore();
